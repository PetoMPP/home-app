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
        let rng = Rng::new(self.rng);
        critical_section::with(|cs| {
            crate::RNG.borrow_ref_mut(cs).replace(rng);
        });
        let init = initialize(
            EspWifiInitFor::Wifi,
            timer,
            rng,
            self.radio_clk,
            self.clocks,
        )
        .unwrap();
        // Configure Wifi
        let wifi = self.wifi;
        let (iface, device, mut controller, sockets) =
            create_network_interface(&init, wifi, WifiStaDevice, storage).unwrap();

        controller.start().unwrap();
        log::info!("Is wifi started: {:?}", controller.is_started());

        let delay = Delay::new(self.clocks);
        log::info!("Start Wifi Scan");
        loop {
            let res: Result<(heapless::Vec<AccessPointInfo, 10>, usize), WifiError> =
                controller.scan_n();
            if let Ok((res, _count)) = res {
                let mut found = false;
                for ap in res {
                    log::info!("{:?}", ap);
                    if ap.ssid == SSID {
                        log::info!("Found AP: {:?}", ap);
                        found = true;
                        controller
                            .set_configuration(&Configuration::Client(ClientConfiguration {
                                ssid: ap.ssid,
                                password: PASSWORD.try_into().unwrap(),
                                auth_method: ap.auth_method.unwrap_or(AuthMethod::WPA2Personal),
                                bssid: Some(ap.bssid),
                                channel: Some(ap.channel),
                            }))
                            .unwrap();
                        break;
                    }
                }
                if found {
                    break;
                }
                log::error!("Scan done, no AP found");
            }
        }

        log::info!("{:?}", controller.get_capabilities());
        log::info!("{:?}", controller.get_configuration());
        log::info!("Wi-Fi connect: {:?}", controller.connect());

        log::info!("Wait to get connected");
        const DELAY_MS: u32 = 10000;
        loop {
            let res = controller.is_connected();
            match res {
                Ok(connected) => {
                    if connected {
                        break;
                    }
                }
                Err(err) => {
                    log::warn!("Error: {:?}, restarting in {}ms", err, DELAY_MS);
                    delay.delay_millis(DELAY_MS);
                    esp_hal::reset::software_reset();
                    unreachable!("Resetting");
                }
            }
        }

        let wifi_stack = WifiStack::new(iface, device, sockets, current_millis);
        let start_time = current_millis();
        log::info!("Wait to get an ip address");
        loop {
            wifi_stack.work();

            if wifi_stack.is_iface_up() {
                log::info!(
                    "got ip {:?} in {}ms",
                    wifi_stack.get_ip_info(),
                    current_millis() - start_time
                );
                break;
            }
        }

        wifi_stack
    }
}
