use esp_hal::assign_resources;

assign_resources! {
    pub Resources<'d> {
        bt: BluetoothResources<'d> {
            bt:  BT,
        },
        rain: RainResources<'d> {
            gpio:  GPIO19,
        },
        light: LightResources<'d> {
            gpio:  GPIO36,
        },
        dht11: DHT11Resources<'d> {
            pin:  GPIO17,
        },
        wind: WindResources<'d> {
            direction:  GPIO35,
            speed:  GPIO34,
        },
    }
}
