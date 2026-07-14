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
    let lon_field = (lon / 20.0) as u32;
    let lat_field = (lat / 10.0) as u32;
    let lon = lon - (lon_field as f64) * 20.0;
    let lat = lat - (lat_field as f64) * 10.0;

    // Square: longitude in 2° cells, latitude in 1° cells (digits 0..9).
    let lon_sq = (lon / 2.0) as u32;
    let lat_sq = lat as u32;

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
    let lon_field = (lon / 20.0) as u32;
    let lat_field = (lat / 10.0) as u32;
    let lon = lon - (lon_field as f64) * 20.0;
    let lat = lat - (lat_field as f64) * 10.0;

    // Square: longitude in 2° cells, latitude in 1° cells (digits 0..9).
    let lon_sq = (lon / 2.0) as u32;
    let lat_sq = lat as u32;
    let lon = lon - (lon_sq as f64) * 2.0;
    let lat = lat - (lat_sq as f64);

    // Subsquare: 24 subdivisions per square (letters a..x).
    let lon_sub = (lon * 12.0) as u32;
    let lat_sub = (lat * 24.0) as u32;

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
