use esp_hal::assign_resources;

assign_resources! {
    pub Resources<'d> {
        bt: BluetoothResources<'d> {
            bt:  BT,
        },
        adc: AdcResources<'d> {
            adc: ADC1,
            light: GPIO34,
            direction: GPIO35,
        },
        dht11: DHT11Resources<'d> {
            pin:  GPIO16,
        },
        dps310: DPS310Resources<'d> {
            i2c: I2C0,
            sda: GPIO21,
            scl: GPIO22,
        },
        rain: RainResources<'d> {
            pin:  GPIO23,
        },
        wind: WindResources<'d> {
            speed:  GPIO27,
        },
    }
}
