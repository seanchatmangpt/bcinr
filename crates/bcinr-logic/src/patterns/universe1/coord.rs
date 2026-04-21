//! # U1Coord — Packed U8 Coordinate
//!
//! Packs three 3-bit axes into a u16: each axis indexes 8 atoms; a full coord selects one atom within a 512-atom block.
//! Format: bits [8:6] = domain, bits [5:3] = cell, bits [2:0] = place.

use super::constants::*;

/// Packed U8 coordinate: `(domain, cell, place)` each in `[0, 7]`.
///
/// ```
/// use bcinr_logic::patterns::universe1::coord::U1Coord;
/// let c = U1Coord::new_const(3, 5, 7);
/// assert_eq!(c.domain(), 3);
/// assert_eq!(c.cell(), 5);
/// assert_eq!(c.place(), 7);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct U1Coord(u16);

impl U1Coord {
    /// Construct from raw packed value (no validation of unused high bits).
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::coord::U1Coord;
    /// let raw = (3u16 << 6) | (5u16 << 3) | 7u16;
    /// let c = U1Coord::from_raw(raw);
    /// assert_eq!(c.domain(), 3);
    /// ```
    #[inline(always)]
    pub const fn from_raw(raw: u16) -> Self {
        Self(raw)
    }

    /// Const constructor — clamps each axis to low 3 bits.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::coord::U1Coord;
    /// const C: U1Coord = U1Coord::new_const(1, 2, 3);
    /// assert_eq!(C.domain(), 1);
    /// assert_eq!(C.cell(), 2);
    /// assert_eq!(C.place(), 3);
    /// ```
    #[inline(always)]
    pub const fn new_const(domain: u8, cell: u8, place: u8) -> Self {
        let d = (domain as u16) & (U1_COORD_MASK as u16);
        let c = (cell as u16) & (U1_COORD_MASK as u16);
        let p = (place as u16) & (U1_COORD_MASK as u16);
        Self((d << U1_DOMAIN_SHIFT) | (c << U1_CELL_SHIFT) | p)
    }

    /// Validated constructor — returns None if any axis ≥ 8.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::coord::U1Coord;
    /// assert!(U1Coord::try_new(7, 7, 7).is_some());
    /// assert!(U1Coord::try_new(8, 0, 0).is_none());
    /// assert!(U1Coord::try_new(0, 8, 0).is_none());
    /// assert!(U1Coord::try_new(0, 0, 8).is_none());
    /// ```
    #[inline(always)]
    pub const fn try_new(domain: u8, cell: u8, place: u8) -> Option<Self> {
        if domain < 8 && cell < 8 && place < 8 {
            Some(Self::new_const(domain, cell, place))
        } else {
            None
        }
    }

    /// Domain axis [0, 7].
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::coord::U1Coord;
    /// assert_eq!(U1Coord::new_const(4, 0, 0).domain(), 4);
    /// ```
    #[inline(always)]
    pub const fn domain(self) -> u8 {
        ((self.0 >> U1_DOMAIN_SHIFT) & (U1_COORD_MASK as u16)) as u8
    }

    /// Cell axis [0, 7].
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::coord::U1Coord;
    /// assert_eq!(U1Coord::new_const(0, 6, 0).cell(), 6);
    /// ```
    #[inline(always)]
    pub const fn cell(self) -> u8 {
        ((self.0 >> U1_CELL_SHIFT) & (U1_COORD_MASK as u16)) as u8
    }

    /// Place axis [0, 7].
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::coord::U1Coord;
    /// assert_eq!(U1Coord::new_const(0, 0, 2).place(), 2);
    /// ```
    #[inline(always)]
    pub const fn place(self) -> u8 {
        (self.0 & (U1_COORD_MASK as u16)) as u8
    }

    /// Flat word index within a U1_4096: `domain * 8 + cell`, in `[0, 63]`.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::coord::U1Coord;
    /// assert_eq!(U1Coord::new_const(3, 5, 0).word_index(), 3 * 8 + 5);
    /// assert_eq!(U1Coord::new_const(7, 7, 0).word_index(), 63);
    /// ```
    #[inline(always)]
    pub const fn word_index(self) -> usize {
        (self.domain() as usize) * 8 + (self.cell() as usize)
    }

    /// Bit index within the selected word: same as `place()` in `[0, 7]`.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::coord::U1Coord;
    /// assert_eq!(U1Coord::new_const(0, 0, 5).bit_index(), 5);
    /// ```
    #[inline(always)]
    pub const fn bit_index(self) -> u32 {
        self.place() as u32
    }

    /// Raw packed `u16` value.
    ///
    /// ```
    /// use bcinr_logic::patterns::universe1::coord::U1Coord;
    /// let c = U1Coord::new_const(1, 2, 3);
    /// let raw = c.as_u16();
    /// assert_eq!(U1Coord::from_raw(raw), c);
    /// ```
    #[inline(always)]
    pub const fn as_u16(self) -> u16 {
        self.0
    }
}
