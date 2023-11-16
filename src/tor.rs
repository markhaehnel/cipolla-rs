use std::str::FromStr;

use arti_client::{BootstrapBehavior, StreamPrefs, TorClient};
use tor_geoip::CountryCode;
use tor_rtcompat::PreferredRuntime;

pub fn build_tor_client(country_code: Option<&str>) -> TorClient<PreferredRuntime> {
    let mut tor_client = TorClient::builder()
        .bootstrap_behavior(BootstrapBehavior::OnDemand)
        .create_unbootstrapped()
        .expect("Failed to create tor client");

    if let Some(country_code) = country_code {
        let cc = CountryCode::from_str(country_code).expect("Country code is invalid");

        let mut stream_prefs = StreamPrefs::default();
        stream_prefs.exit_country(cc);

        tor_client.set_stream_prefs(stream_prefs);
    }

    tor_client
}
