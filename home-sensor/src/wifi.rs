use esp_hal::clock::Clocks;
use esp_hal::delay::Delay;
use esp_hal::peripherals::{RADIO_CLK, RNG, SYSTIMER, WIFI};
use esp_hal::rng::Rng;
use esp_hal::timer::systimer::SystemTimer;
use esp_wifi::wifi::utils::create_network_interface;
use esp_wifi::wifi::{AccessPointInfo, AuthMethod, ClientConfiguration, Configuration};
use esp_wifi::wifi::{WifiError, WifiStaDevice};
use esp_wifi::wifi_interface::WifiStack;
use esp_wifi::{current_millis, initialize, EspWifiInitFor};
use smoltcp::iface::SocketStorage;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("WIFI_PASSWORD");

#[macro_export]
macro_rules! wifi_builder {
    ($peripherals:expr, $clocks:expr) => {
        wifi::WifiConnectionBuilder::new(
            $peripherals.SYSTIMER,
            $peripherals.RNG,
            $peripherals.RADIO_CLK,
            &$clocks,
            $peripherals.WIFI,
        )
    };
}

pub struct WifiConnectionBuilder<'d> {
    sys_timer: SYSTIMER,
    rng: RNG,
    radio_clk: RADIO_CLK,
    clocks: &'d Clocks<'d>,
    wifi: WIFI,
}

impl WifiConnectionBuilder<'_> {
    pub fn new<'d>(
        sys_timer: SYSTIMER,
        rng: RNG,
        radio_clk: RADIO_CLK,
        clocks: &'d Clocks<'d>,
        wifi: WIFI,
    ) -> WifiConnectionBuilder<'d> {
        WifiConnectionBuilder {
            sys_timer,
            rng,
            radio_clk,
            clocks,
            wifi,
        }
    }

    pub fn connect<'a>(
        self,
        storage: &'a mut [SocketStorage<'a>; 3],
    ) -> WifiStack<'a, WifiStaDevice> {
        // Initialize the timers used for Wifi
        let timer = SystemTimer::new(self.sys_timer).alarm0;
        let init = initialize(
            EspWifiInitFor::Wifi,
            timer,
            Rng::new(self.rng),
            self.radio_clk,
            self.clocks,
        )
        .unwrap();
        // Configure Wifi
        let wifi = self.wifi;
        let (iface, device, mut controller, sockets) =
            create_network_interface(&init, wifi, WifiStaDevice, storage).unwrap();

        let mut auth_method = AuthMethod::WPA2Personal;
        if PASSWORD.is_empty() {
            auth_method = AuthMethod::None;
        }

        let client_config = Configuration::Client(ClientConfiguration {
            ssid: SSID.try_into().unwrap(),
            password: PASSWORD.try_into().unwrap(),
            auth_method,
            ..Default::default()
        });

        let res = controller.set_configuration(&client_config);
        log::info!("Wi-Fi set_configuration returned {:?}", res);

        controller.start().unwrap();
        log::info!("Is wifi started: {:?}", controller.is_started());

        log::info!("Start Wifi Scan");
        let res: Result<(heapless::Vec<AccessPointInfo, 10>, usize), WifiError> =
            controller.scan_n();
        if let Ok((res, _count)) = res {
            for ap in res {
                log::info!("{:?}", ap);
            }
        }

        log::info!("{:?}", controller.get_capabilities());
        log::info!("Wi-Fi connect: {:?}", controller.connect());

        log::info!("Wait to get connected");
        let delay = Delay::new(self.clocks);
        const RETRY_DELAY_MS: u32 = 15000;
        loop {
            let res = controller.is_connected();
            match res {
                Ok(connected) => {
                    if connected {
                        break;
                    }
                }
                Err(err) => {
                    log::warn!("{:?}, retry in {}s..", err, RETRY_DELAY_MS / 1000);
                    delay.delay_millis(RETRY_DELAY_MS);
                    log::info!("Wi-Fi reconnect: {:?}", controller.connect());
                }
            }
        }
        log::info!("{:?}", controller.is_connected());

        let wifi_stack = WifiStack::new(iface, device, sockets, current_millis);
        log::info!("Wait to get an ip address");
        loop {
            wifi_stack.work();

            if wifi_stack.is_iface_up() {
                log::info!("got ip {:?}", wifi_stack.get_ip_info());
                break;
            }
        }

        wifi_stack
    }
}
