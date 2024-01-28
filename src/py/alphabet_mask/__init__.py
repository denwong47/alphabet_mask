# -*- coding: utf-8 -*-
"""
================================
 alphabet_mask
================================

Create alphabet masks from strings.

This project includes a Rust binary backend:

- :mod:`lib_alphabet_mask` which can be loaded as
  :attr:`~alphabet_mask.bin`.
"""
__all__ = [
    "python",
    "rust",
]
from . import python, lib_alphabet_mask as rust
