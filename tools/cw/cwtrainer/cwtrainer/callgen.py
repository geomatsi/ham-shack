"""Realistic-looking callsign generator."""

from __future__ import annotations

import random
import string
from dataclasses import dataclass

ONE_LETTER = ["G", "K", "W", "N", "F", "I", "M", "R"]
TWO_LETTER = ["OH", "SM", "DL", "UA", "JA", "VK", "OK", "SP", "EA", "PA", "ON", "HA", "LY",
              "YL", "ES", "OE", "HB", "VE", "ZL", "LA", "OZ", "EI", "YO", "LZ", "UR", "RA",
              "KA", "WA", "DJ", "DK", "IK", "OM", "EU", "CT"]
DIGIT_PREFIX = ["S5", "9A", "4X", "E7", "Z3", "4Z", "5B", "9H", "3A", "4L", "T7", "Z6"]
PORTABLE = ["/P", "/M", "/QRP"]


@dataclass(frozen=True)
class Pattern:
    prefix: int  # letters before the area digit
    suffix: int  # letters after it

    def __str__(self) -> str:
        return f"{self.prefix}x{self.suffix}"


# easiest first; difficulty levels unlock them in this order
PATTERNS = [Pattern(1, 2), Pattern(2, 1), Pattern(1, 3), Pattern(2, 2), Pattern(2, 3)]
MAX_LEVEL = 5


class CallsignGenerator:
    def __init__(self, rng: random.Random | None = None, portable: float = 0.1,
                 digit_prefix: float = 0.1) -> None:
        self.rng = rng or random.Random()
        self.portable = portable
        self.digit_prefix = digit_prefix

    def generate(self, level: int = MAX_LEVEL, pattern: Pattern | None = None) -> str:
        """level 1: 1x2/2x1; 2: +1x3/2x2; 3: +2x3; 4: +digit prefixes; 5: +/P /M /QRP."""
        rng = self.rng
        if pattern is None:
            pattern = rng.choice(PATTERNS[:2 * level])
        if pattern.prefix == 2 and level >= 4 and rng.random() < self.digit_prefix:
            prefix = rng.choice(DIGIT_PREFIX)
        else:
            prefix = rng.choice(ONE_LETTER if pattern.prefix == 1 else TWO_LETTER)
        call = prefix + rng.choice(string.digits) + "".join(
            rng.choice(string.ascii_uppercase) for _ in range(pattern.suffix))
        if level >= 5 and rng.random() < self.portable:
            call += rng.choice(PORTABLE)
        return call
