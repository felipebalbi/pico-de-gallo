//! ADC endpoint handlers.

#[cfg(feature = "hw-rev2")]
use defmt::debug;
#[cfg(feature = "hw-rev2")]
use pico_de_gallo_internal::{
    ADC_NOMINAL_REFERENCE_MV, ADC_RESOLUTION_BITS, AdcChannel, AdcConfigurationInfo, NUM_ADC_GPIO_CHANNELS,
};
use pico_de_gallo_internal::{AdcError, AdcGetConfigurationResponse, AdcReadRequest, AdcReadResponse};
use postcard_rpc::header::VarHeader;

use crate::context::Context;

/// Map an [`AdcChannel`] variant to the index into `context.adc_channels`.
#[cfg(feature = "hw-rev2")]
fn adc_channel_index(channel: AdcChannel) -> usize {
    match channel {
        AdcChannel::Adc0 => 0,
        AdcChannel::Adc1 => 1,
        AdcChannel::Adc2 => 2,
        AdcChannel::Adc3 => 3,
    }
}

/// Handler for `adc/read` — single-shot ADC read returning a raw 12-bit value.
#[cfg(feature = "hw-rev2")]
pub(crate) fn adc_read_handler(context: &mut Context, _header: VarHeader, req: AdcReadRequest) -> AdcReadResponse {
    let idx = adc_channel_index(req.channel);
    let ch = &mut context.adc_channels[idx];

    match context.adc.blocking_read(ch) {
        Ok(raw) => {
            debug!("adc read: ch={=usize} raw={=u16}", idx, raw);
            Ok(raw)
        }
        Err(_) => Err(AdcError::ConversionFailed),
    }
}

#[cfg(not(feature = "hw-rev2"))]
pub(crate) fn adc_read_handler(_context: &mut Context, _header: VarHeader, _req: AdcReadRequest) -> AdcReadResponse {
    Err(AdcError::Unsupported)
}

/// Handler for `adc/get-config` — returns ADC configuration info.
#[cfg(feature = "hw-rev2")]
pub(crate) fn adc_get_config_handler(
    _context: &mut Context,
    _header: VarHeader,
    _req: (),
) -> AdcGetConfigurationResponse {
    debug!("adc get config");
    Ok(AdcConfigurationInfo {
        resolution_bits: ADC_RESOLUTION_BITS,
        nominal_reference_mv: ADC_NOMINAL_REFERENCE_MV,
        num_gpio_channels: NUM_ADC_GPIO_CHANNELS as u8,
    })
}

#[cfg(not(feature = "hw-rev2"))]
pub(crate) fn adc_get_config_handler(
    _context: &mut Context,
    _header: VarHeader,
    _req: (),
) -> AdcGetConfigurationResponse {
    Err(AdcError::Unsupported)
}
