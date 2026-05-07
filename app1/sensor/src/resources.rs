use esp_hal::assign_resources;

assign_resources! {
    pub Resources<'d> {
        bt: BluetoothResources<'d> {
            bt:  BT,
        },
        // rain: RainResources<'d> {
        //     gpio:  GPIO19,
        // },
        dht11: DHT11Resources<'d> {
            pin:  GPIO16,
        },
        dps310: DPS310Resources<'d> {
            i2c: I2C0,
            sda: GPIO21,
            scl: GPIO22,
        },
        light: LightResources<'d> {
            adc:  ADC1,
            pin: GPIO34,
        },
        // wind: WindResources<'d> {
        //     direction:  GPIO35,
        //     speed:  GPIO34,
        // },
    }
}
