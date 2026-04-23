//! PWM endpoint handlers.

use defmt::debug;
use fixed::traits::ToFixed;
use pico_de_gallo_internal::{
    NUM_PWM_CHANNELS, PwmConfigurationInfo, PwmDisableRequest, PwmDisableResponse, PwmDutyCycleInfo, PwmEnableRequest,
    PwmEnableResponse, PwmError, PwmGetConfigurationRequest, PwmGetConfigurationResponse, PwmGetDutyCycleRequest,
    PwmGetDutyCycleResponse, PwmSetConfigurationRequest, PwmSetConfigurationResponse, PwmSetDutyCycleRequest,
    PwmSetDutyCycleResponse,
};
use postcard_rpc::header::VarHeader;

use crate::context::{Context, SYS_CLK_HZ};

/// Returns the (slice_index, is_channel_b) pair for a PWM channel number.
///
/// Channel 0 → slice 0, channel A
/// Channel 1 → slice 0, channel B
/// Channel 2 → slice 1, channel A
/// Channel 3 → slice 1, channel B
fn pwm_channel_parts(channel: u8) -> Result<(usize, bool), PwmError> {
    if channel as usize >= NUM_PWM_CHANNELS {
        return Err(PwmError::InvalidChannel);
    }
    let slice_idx = usize::from(channel) / 2;
    let is_b = !channel.is_multiple_of(2);
    Ok((slice_idx, is_b))
}

/// Compute `top` and integer divider from a target frequency.
///
/// For non-phase-correct:  `f_pwm = f_sys / (divider * (top + 1))`
/// For phase-correct:      `f_pwm = f_sys / (2 * divider * top)`
fn compute_pwm_params(freq_hz: u32, phase_correct: bool) -> Result<(u16, u16), PwmError> {
    if freq_hz == 0 {
        return Err(PwmError::InvalidConfiguration);
    }

    (1u32..=4095)
        .find_map(|div| {
            let top = if phase_correct {
                // f = sys / (2 * div * top), so top = sys / (2 * div * f)
                let denom = 2u64 * u64::from(div) * u64::from(freq_hz);
                u64::from(SYS_CLK_HZ).checked_div(denom)
            } else {
                // f = sys / (div * (top + 1)), so top = sys / (div * f) - 1
                let denom = u64::from(div) * u64::from(freq_hz);
                u64::from(SYS_CLK_HZ)
                    .checked_div(denom)
                    .and_then(|raw| raw.checked_sub(1))
            }?;

            (top > 0 && top <= u64::from(u16::MAX)).then_some((top as u16, div as u16))
        })
        .ok_or(PwmError::InvalidConfiguration)
}

/// Handler for `pwm/set-duty-cycle` — sets the duty cycle of a PWM channel.
pub(crate) fn pwm_set_duty_cycle_handler(
    context: &mut Context,
    _header: VarHeader,
    req: PwmSetDutyCycleRequest,
) -> PwmSetDutyCycleResponse {
    let (slice_idx, is_b) = pwm_channel_parts(req.channel)?;
    let top = context.pwm_configs[slice_idx].top;

    // Clamp duty to top+1 (which means always-on)
    let compare = req.duty.min(top.saturating_add(1));

    if is_b {
        context.pwm_configs[slice_idx].compare_b = compare;
    } else {
        context.pwm_configs[slice_idx].compare_a = compare;
    }

    context.pwm_slices[slice_idx].set_config(&context.pwm_configs[slice_idx]);
    debug!(
        "pwm set duty: ch={=u8} compare={=u16} top={=u16}",
        req.channel, compare, top
    );
    Ok(())
}

/// Handler for `pwm/get-duty-cycle` — returns the current duty cycle info.
pub(crate) fn pwm_get_duty_cycle_handler(
    context: &mut Context,
    _header: VarHeader,
    req: PwmGetDutyCycleRequest,
) -> PwmGetDutyCycleResponse {
    let (slice_idx, is_b) = pwm_channel_parts(req.channel)?;
    let cfg = &context.pwm_configs[slice_idx];
    let compare = if is_b { cfg.compare_b } else { cfg.compare_a };
    // max_duty_cycle is top + 1 (full scale)
    let max_duty = cfg.top.saturating_add(1);

    debug!(
        "pwm get duty: ch={=u8} compare={=u16} max={=u16}",
        req.channel, compare, max_duty
    );

    Ok(PwmDutyCycleInfo {
        current_duty: compare,
        max_duty,
    })
}

/// Handler for `pwm/enable` — enables a PWM slice (identified by channel).
pub(crate) fn pwm_enable_handler(
    context: &mut Context,
    _header: VarHeader,
    req: PwmEnableRequest,
) -> PwmEnableResponse {
    let (slice_idx, _) = pwm_channel_parts(req.channel)?;
    context.pwm_configs[slice_idx].enable = true;
    context.pwm_slices[slice_idx].set_config(&context.pwm_configs[slice_idx]);
    debug!("pwm enable: ch={=u8} slice={=usize}", req.channel, slice_idx);
    Ok(())
}

/// Handler for `pwm/disable` — disables a PWM slice (identified by channel).
pub(crate) fn pwm_disable_handler(
    context: &mut Context,
    _header: VarHeader,
    req: PwmDisableRequest,
) -> PwmDisableResponse {
    let (slice_idx, _) = pwm_channel_parts(req.channel)?;
    context.pwm_configs[slice_idx].enable = false;
    context.pwm_slices[slice_idx].set_config(&context.pwm_configs[slice_idx]);
    debug!("pwm disable: ch={=u8} slice={=usize}", req.channel, slice_idx);
    Ok(())
}

/// Handler for `pwm/set-config` — configures PWM frequency and phase-correct mode.
pub(crate) fn pwm_set_config_handler(
    context: &mut Context,
    _header: VarHeader,
    req: PwmSetConfigurationRequest,
) -> PwmSetConfigurationResponse {
    let (slice_idx, _) = pwm_channel_parts(req.channel)?;
    let (top, div) = compute_pwm_params(req.frequency_hz, req.phase_correct)?;

    // Preserve existing compare values (scaled to new top if needed)
    let old_cfg = &context.pwm_configs[slice_idx];
    let old_top = old_cfg.top;
    let old_a = old_cfg.compare_a;
    let old_b = old_cfg.compare_b;

    // Scale compare values proportionally to the new top
    let new_a = if old_top == 0 {
        0
    } else {
        ((u32::from(old_a) * u32::from(top)) / u32::from(old_top)) as u16
    };
    let new_b = if old_top == 0 {
        0
    } else {
        ((u32::from(old_b) * u32::from(top)) / u32::from(old_top)) as u16
    };

    context.pwm_configs[slice_idx].top = top;
    context.pwm_configs[slice_idx].divider = div.to_fixed();
    context.pwm_configs[slice_idx].phase_correct = req.phase_correct;
    context.pwm_configs[slice_idx].compare_a = new_a;
    context.pwm_configs[slice_idx].compare_b = new_b;
    context.pwm_slices[slice_idx].set_config(&context.pwm_configs[slice_idx]);

    debug!(
        "pwm set config: ch={=u8} freq={=u32} top={=u16} div={=u16} pc={=bool}",
        req.channel, req.frequency_hz, top, div, req.phase_correct
    );
    Ok(())
}

/// Handler for `pwm/get-config` — returns the current PWM configuration.
pub(crate) fn pwm_get_config_handler(
    context: &mut Context,
    _header: VarHeader,
    req: PwmGetConfigurationRequest,
) -> PwmGetConfigurationResponse {
    let (slice_idx, _) = pwm_channel_parts(req.channel)?;
    let cfg = &context.pwm_configs[slice_idx];

    // Reconstruct frequency from top/divider/phase_correct
    let div_int: u32 = cfg.divider.to_bits() as u32 >> 4; // integer part of 12.4 fixed
    let div_val = if div_int == 0 { 1u32 } else { div_int };

    let frequency_hz = if cfg.phase_correct {
        // f = sys / (2 * div * top)
        let denom = 2u64 * u64::from(div_val) * u64::from(cfg.top);
        u64::from(SYS_CLK_HZ).checked_div(denom).unwrap_or(0) as u32
    } else {
        // f = sys / (div * (top + 1))
        let denom = u64::from(div_val) * (u64::from(cfg.top) + 1);
        u64::from(SYS_CLK_HZ).checked_div(denom).unwrap_or(0) as u32
    };

    debug!(
        "pwm get config: ch={=u8} freq={=u32} pc={=bool} en={=bool}",
        req.channel, frequency_hz, cfg.phase_correct, cfg.enable
    );

    Ok(PwmConfigurationInfo {
        frequency_hz,
        phase_correct: cfg.phase_correct,
        enabled: cfg.enable,
    })
}
