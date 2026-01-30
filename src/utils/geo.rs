use std::f64::consts::PI;

const EARTH_RADIUS_KM: f64 = 6371.0;

/// ハヴァサイン公式を使用して2地点間の距離を計算（メートル単位で返す）
pub fn haversine_distance(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
  let lat1_rad = lat1 * PI / 180.0;
  let lat2_rad = lat2 * PI / 180.0;
  let delta_lat = (lat2 - lat1) * PI / 180.0;
  let delta_lng = (lng2 - lng1) * PI / 180.0;

  let a = (delta_lat / 2.0).sin().powi(2) + lat1_rad.cos() * lat2_rad.cos() * (delta_lng / 2.0).sin().powi(2);

  let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

  EARTH_RADIUS_KM * c * 1000.0 // km を m に変換
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_haversine_same_location() {
    let distance = haversine_distance(35.6812, 139.7671, 35.6812, 139.7671);
    assert!(distance < 0.1);
  }

  #[test]
  fn test_haversine_tokyo_to_osaka() {
    // 東京タワー -> 大阪城
    let distance = haversine_distance(35.6812, 139.7671, 34.6937, 135.5023);
    // 約 400km (400,000m)
    assert!(distance > 390000.0 && distance < 410000.0);
  }

  #[test]
  fn test_haversine_tokyo_to_sapporo() {
    // 東京 -> 札幌
    let distance = haversine_distance(35.6812, 139.7671, 43.0642, 141.3469);
    // 約 830km (830,000m)
    assert!(distance > 810000.0 && distance < 850000.0);
  }

  #[test]
  fn test_haversine_short_distance() {
    // 東京タワー -> 東京駅 (約4km)
    let distance = haversine_distance(35.6812, 139.7671, 35.6812, 139.7671);
    assert!(distance < 10000.0);
  }

  #[test]
  fn test_haversine_negative_coordinates() {
    // ロンドン -> ニューヨーク
    let distance = haversine_distance(51.5074, -0.1278, 40.7128, -74.0060);
    // 約 5,570km
    assert!(distance > 5500000.0 && distance < 5600000.0);
  }
}
