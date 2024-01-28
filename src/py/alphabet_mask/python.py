# -*- coding: utf-8 -*-
"""
Python version.
"""
from typing import Set, Iterable


def alphabet_mask(string: str) -> int:
    """
    Returns a bit mask representing the alphabet letters in the given string.

    Masked characters are:
    - space (#0)
    - A-Z (case insensitive) (#1-26)
    - full stop (#27)
    - comma (#28)
    """
    mask = 0
    for char in string:
        char_code = ord(char)

        if char_code == 32:
            mask |= 1 << 0
        elif char_code == 46:
            mask |= 1 << 27
        elif char_code == 44:
            mask |= 1 << 28
        elif char_code == 39:
            mask |= 1 << 29
        elif char_code == 45:
            mask |= 1 << 30
        elif char_code == 34:
            mask |= 1 << 31
        elif (char_code & 64) and not (char_code & 128):
            mask |= 1 << (char_code & 31)
        else:
            raise ValueError(f"Invalid character: {char}")

    return mask


def alphabet_set(string: str) -> Set[str]:
    """
    Simply returns a set of the alphabet letters in the given string.

    For speed comparisons only.
    """
    return set(string.lower())


def mask_to_chars(mask: int) -> str:
    """
    Convert a mask created by :func:`alphabet_mask` to a string of characters.
    """
    chars = ""
    for i in range(32):
        if mask & (1 << i):
            _mapper = {
                0: " ",
                27: ".",
                28: ",",
                29: "'",
                30: "-",
                31: '"',
            }
            chars += _mapper.get(i, chr(i + 96))

    return chars


def intersect_masks(masks: Iterable[int]) -> int:
    """
    Return the union of a list of masks.
    """
    mask = (1 << 32) - 1
    for m in masks:
        mask &= m
    return mask


def find_common_mask(strings: Iterable[str]) -> int:
    """
    Return the common mask of a list of strings.
    """
    masks = (alphabet_mask(s) for s in strings)
    return intersect_masks(masks)


def common_alphabets(strings: Iterable[str]) -> Set[str]:
    """
    Return the common alphabet of a list of strings.
    """
    mask = find_common_mask(strings)
    return mask_to_chars(mask)
