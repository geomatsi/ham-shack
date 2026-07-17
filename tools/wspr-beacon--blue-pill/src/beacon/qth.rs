//! Agent: Claude Opus-4.8
//!
//! GPS coordinate utilities (`no_std`).
//!
//! Converts a WGS84 GPS position into a Maidenhead (QTH) locator.
//!
//! Input is plain signed decimal degrees ([`Coordinates`]). GPS/NMEA-0183 fixes
//! are already referenced to the WGS84 datum, which is also the datum the
//! Maidenhead locator system is defined against, so no datum transformation is
//! performed here.
//!
//! This crate is deliberately independent of any NMEA parser. When driving it
//! from the [`nmea0183`](https://github.com/nsforth/nmea0183) crate (e.g. with a
//! u-blox NEO-7M), convert at the call site:
//!
//! ```ignore
//! let coord = Coordinates {
//!     latitude: latitude.as_f64(),   // nmea0183::coords::Latitude
//!     longitude: longitude.as_f64(), // nmea0183::coords::Longitude
//! };
//! ```
//!
//! Both functions validate the coordinate range and return a `&str` view into a
//! caller-supplied byte buffer, so no allocation is required.
//!
//! Naming follows the official Maidenhead precision levels:
//! field (2 chars) → *square* (4 chars) → *subsquare* (6 chars).

/// A WGS84 GPS position in signed decimal degrees.
///
/// Named fields (rather than a bare `(f64, f64)`) prevent accidentally swapping
/// latitude and longitude at the call site.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinates {
    /// Latitude in decimal degrees, `-90.0..=90.0` (negative = South).
    pub latitude: f64,
    /// Longitude in decimal degrees, `-180.0..=180.0` (negative = West).
    pub longitude: f64,
}

/// Error returned when a coordinate is outside the valid GPS range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QthError {
    /// Latitude was outside `-90.0..=90.0`.
    LatitudeOutOfRange,
    /// Longitude was outside `-180.0..=180.0`.
    LongitudeOutOfRange,
}

/// Reject coordinates that fall outside the valid GPS range.
fn validate(coord: Coordinates) -> Result<(), QthError> {
    if !(-90.0..=90.0).contains(&coord.latitude) {
        return Err(QthError::LatitudeOutOfRange);
    }
    if !(-180.0..=180.0).contains(&coord.longitude) {
        return Err(QthError::LongitudeOutOfRange);
    }
    Ok(())
}

/// Coarse conversion: GPS coordinates → 4-character Maidenhead square, e.g. `"JN58"`.
///
/// Writes into `buf` and returns a `&str` borrowing it. Returns [`QthError`] if
/// the coordinate is out of range.
pub fn qth_square(coord: Coordinates, buf: &mut [u8; 4]) -> Result<&str, QthError> {
    validate(coord)?;

    // Shift into positive ranges: longitude 0..360, latitude 0..180.
    let lon = coord.longitude + 180.0;
    let lat = coord.latitude + 90.0;

    // Field: longitude in 20° cells, latitude in 10° cells (letters A..R).
    // Clamp to the last valid index (R = 17): at the exact upper edge
    // (lon == 180.0 or lat == 90.0) the division yields 18, which would emit 'S',
    // one past the Maidenhead range. `validate()` accepts those edge values, so
    // the clamp is what keeps the output in range.
    let lon_field = ((lon / 20.0) as u32).min(17);
    let lat_field = ((lat / 10.0) as u32).min(17);
    let lon = lon - (lon_field as f64) * 20.0;
    let lat = lat - (lat_field as f64) * 10.0;

    // Square: longitude in 2° cells, latitude in 1° cells (digits 0..9).
    // Clamp likewise: at the upper edge the clamped field leaves a full residual,
    // so the raw index would be 10. The corner then encodes as "RR99".
    let lon_sq = ((lon / 2.0) as u32).min(9);
    let lat_sq = (lat as u32).min(9);

    *buf = [
        b'A' + lon_field as u8,
        b'A' + lat_field as u8,
        b'0' + lon_sq as u8,
        b'0' + lat_sq as u8,
    ];

    // Every byte is ASCII by construction, so this never fails.
    Ok(core::str::from_utf8(&buf[..]).unwrap())
}

/// Fine conversion: GPS coordinates → 6-character Maidenhead subsquare, e.g. `"JN58td"`.
///
/// Writes into `buf` and returns a `&str` borrowing it. Returns [`QthError`] if
/// the coordinate is out of range.
pub fn qth_subsquare(coord: Coordinates, buf: &mut [u8; 6]) -> Result<&str, QthError> {
    validate(coord)?;

    // Shift into positive ranges: longitude 0..360, latitude 0..180.
    let lon = coord.longitude + 180.0;
    let lat = coord.latitude + 90.0;

    // Field: longitude in 20° cells, latitude in 10° cells (letters A..R).
    // Clamp to the last valid index (R = 17): at the exact upper edge
    // (lon == 180.0 or lat == 90.0) the division yields 18, which would emit 'S',
    // one past the Maidenhead range. `validate()` accepts those edge values, so
    // the clamp is what keeps the output in range.
    let lon_field = ((lon / 20.0) as u32).min(17);
    let lat_field = ((lat / 10.0) as u32).min(17);
    let lon = lon - (lon_field as f64) * 20.0;
    let lat = lat - (lat_field as f64) * 10.0;

    // Square: longitude in 2° cells, latitude in 1° cells (digits 0..9).
    // Clamp to 9 for the same reason (the clamped field leaves a full residual).
    let lon_sq = ((lon / 2.0) as u32).min(9);
    let lat_sq = (lat as u32).min(9);
    let lon = lon - (lon_sq as f64) * 2.0;
    let lat = lat - (lat_sq as f64);

    // Subsquare: 24 subdivisions per square (letters a..x).
    // Clamp to the last valid index (x = 23); the cascade makes the corner "RR99xx".
    let lon_sub = ((lon * 12.0) as u32).min(23);
    let lat_sub = ((lat * 24.0) as u32).min(23);

    *buf = [
        b'A' + lon_field as u8,
        b'A' + lat_field as u8,
        b'0' + lon_sq as u8,
        b'0' + lat_sq as u8,
        b'a' + lon_sub as u8,
        b'a' + lat_sub as u8,
    ];

    // Every byte is ASCII by construction, so this never fails.
    Ok(core::str::from_utf8(&buf[..]).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subsquare_cities() {
        // (name, latitude, longitude, expected)
        let cases: [(&str, f64, f64, &str); 15] = [
            ("Munich", 48.14666, 11.60833, "JN58td"),
            ("Cairo", 30.0444, 31.2357, "KM50ob"),
            ("Tokyo", 35.6895, 139.6917, "PM95uq"),
            ("New York", 40.7128, -74.0060, "FN20xr"),
            ("Rio de Janeiro", -22.9068, -43.1729, "GG87jc"),
            ("Sydney", -33.8688, 151.2093, "QF56od"),
            ("McMurdo", -77.8419, 166.6863, "RB32id"),
            ("London", 51.5074, -0.1278, "IO91wm"),
            ("Edinburgh", 55.9533, -3.1883, "IO85jw"),
            ("St. Petersburg", 59.9343, 30.3351, "KO59ew"),
            ("Hong Kong", 22.3193, 114.1694, "OL72ch"),
            ("Beijing", 39.9042, 116.4074, "OM89ev"),
            ("Melbourne", -37.8136, 144.9631, "QF22le"),
            ("Santiago", -33.4489, -70.6693, "FF46pn"),
            ("Toronto", 43.6532, -79.3832, "FN03hp"),
        ];

        for (name, latitude, longitude, expected) in cases {
            let mut buf = [0u8; 6];
            let got = qth_subsquare(
                Coordinates {
                    latitude,
                    longitude,
                },
                &mut buf,
            )
            .unwrap();
            assert_eq!(got, expected, "subsquare mismatch for {name}");
        }
    }

    #[test]
    fn square_cities() {
        let cases: [(&str, f64, f64, &str); 15] = [
            ("Munich", 48.14666, 11.60833, "JN58"),
            ("Cairo", 30.0444, 31.2357, "KM50"),
            ("Tokyo", 35.6895, 139.6917, "PM95"),
            ("New York", 40.7128, -74.0060, "FN20"),
            ("Rio de Janeiro", -22.9068, -43.1729, "GG87"),
            ("Sydney", -33.8688, 151.2093, "QF56"),
            ("McMurdo", -77.8419, 166.6863, "RB32"),
            ("London", 51.5074, -0.1278, "IO91"),
            ("Edinburgh", 55.9533, -3.1883, "IO85"),
            ("St. Petersburg", 59.9343, 30.3351, "KO59"),
            ("Hong Kong", 22.3193, 114.1694, "OL72"),
            ("Beijing", 39.9042, 116.4074, "OM89"),
            ("Melbourne", -37.8136, 144.9631, "QF22"),
            ("Santiago", -33.4489, -70.6693, "FF46"),
            ("Toronto", 43.6532, -79.3832, "FN03"),
        ];

        for (name, latitude, longitude, expected) in cases {
            let mut buf = [0u8; 4];
            let got = qth_square(
                Coordinates {
                    latitude,
                    longitude,
                },
                &mut buf,
            )
            .unwrap();
            assert_eq!(got, expected, "square mismatch for {name}");
        }
    }

    #[test]
    fn boundary_corners() {
        // The extreme corners are accepted by `validate()`, so the encoder must
        // keep them inside the Maidenhead range. Before clamping, the upper edge
        // overflowed the field index to 18 ('S', one past 'R').
        let mut buf4 = [0u8; 4];
        let mut buf6 = [0u8; 6];

        // Lower corner: origin of the grid.
        assert_eq!(
            qth_square(
                Coordinates {
                    latitude: -90.0,
                    longitude: -180.0
                },
                &mut buf4
            ),
            Ok("AA00"),
        );
        assert_eq!(
            qth_subsquare(
                Coordinates {
                    latitude: -90.0,
                    longitude: -180.0
                },
                &mut buf6
            ),
            Ok("AA00aa"),
        );

        // Upper corner: the value that used to overflow to 'S'.
        assert_eq!(
            qth_square(
                Coordinates {
                    latitude: 90.0,
                    longitude: 180.0
                },
                &mut buf4
            ),
            Ok("RR99"),
        );
        assert_eq!(
            qth_subsquare(
                Coordinates {
                    latitude: 90.0,
                    longitude: 180.0
                },
                &mut buf6
            ),
            Ok("RR99xx"),
        );

        // Just inside the upper edge lands in the same last cell.
        assert_eq!(
            qth_subsquare(
                Coordinates {
                    latitude: 89.999,
                    longitude: 179.999
                },
                &mut buf6
            ),
            Ok("RR99xx"),
        );
    }

    #[test]
    fn rejects_out_of_range() {
        let mut buf6 = [0u8; 6];
        assert_eq!(
            qth_subsquare(
                Coordinates {
                    latitude: 91.0,
                    longitude: 0.0
                },
                &mut buf6
            ),
            Err(QthError::LatitudeOutOfRange),
        );
        assert_eq!(
            qth_subsquare(
                Coordinates {
                    latitude: 0.0,
                    longitude: -181.0
                },
                &mut buf6
            ),
            Err(QthError::LongitudeOutOfRange),
        );

        let mut buf4 = [0u8; 4];
        assert_eq!(
            qth_square(
                Coordinates {
                    latitude: -90.5,
                    longitude: 0.0
                },
                &mut buf4
            ),
            Err(QthError::LatitudeOutOfRange),
        );
        assert_eq!(
            qth_square(
                Coordinates {
                    latitude: 0.0,
                    longitude: 200.0
                },
                &mut buf4
            ),
            Err(QthError::LongitudeOutOfRange),
        );
    }
}
