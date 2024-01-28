# -*- coding: utf-8 -*-
from typing import Callable
import pytest
from alphabet_mask import python, rust


@pytest.mark.parametrize(
    "mask",
    [
        rust.alphabet_mask,
        python.alphabet_mask,
    ],
)
@pytest.mark.parametrize(
    ("input", "expected"),
    [
        (
            "The quick brown fox jumps over the lazy dog.",
            0b111_11111_11111_11111_11111_11111,  # 28 bits, including space and full stop
        ),
        (
            "a",
            0b10,
        ),
        (
            "",
            0b0,
        ),
        (
            "A",
            0b10,
        ),
        (
            " ",
            0b1,
        ),
        (
            "aA",
            0b10,
        ),
        (
            "a c e",
            0b101011,
        ),
        (
            "a c e.",
            0b100_00000_00000_00000_00001_01011,
        ),
    ],
)
def test_masks(mask: Callable[[str], int], input: str, expected: int):
    """
    Assert that the alphabet mask is correct.
    """
    assert mask(input) == expected
