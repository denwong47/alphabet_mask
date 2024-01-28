# -*- coding: utf-8 -*-
"""
=========================
 Environment Definitions
=========================
Environment settings, mostly relating to a temporary non-production state e.g.
``pytest`` or ``sphinx`` build.
"""
import logging
import os
from typing import Any, Callable, Literal


def get(key: str, modifier: Callable[[str], Any], *, default: Any = None) -> Any:
    """
    Fetch an environment variable, then run it through the modifier.

    Typically he modifier is to change its type, e.g. :class:`int`, :class:`bool` etc.

    Parameters
    ----------
    key : str
        The environment variable name to get.

    modifier : Callable[[str], Any]
        A Callable to transform the found value. Useful for type coersion, since
        environment variables are always stored as :class:`str`.

        .. note::
            If ``modifier`` is ``bool``, special cases apply:

            - Any case-insensitive string value ``"true"`` is considered ``True``.
            - Any case-insensitive string value ``"false"`` is considered ``False``.
            - String values containing only ``0-9`` numerics, then the number is
              converted to :class:`int` before applying :class:`bool`.

    default : Any
        The default value if not found, or modifier encountered an :class:`Exception`.

    Returns
    -------
    Any
        The resultant value.
    """
    _value = os.environ.get(key, default=default)

    if modifier is bool:

        def modifier(value: str) -> bool:
            """
            Special bool transform.
            """
            value = value.strip()

            if value.lower() == "true":
                return True

            if value.lower() == "false":
                return False

            if value.isnumeric():
                return bool(int(value))

            return bool(value)

    if modifier is logging:

        def modifier(value: str) -> int:
            """
            Logging Level transformation.
            """
            value = value.strip().upper()

            return getattr(logging, value, logging.INFO)

    if callable(modifier):
        try:
            return modifier(_value)
        except Exception:
            return default

    return _value


PYTEST_IS_RUNNING = get("PYTEST_RUNNING", bool, default=False)
"""
``True`` if pytest is running, otherwise ``False``.

Set Environment Variable ``PYTEST_RUNNING`` to change this value manually.
"""

SPHINX_IS_BUILDING = get("SPHINX_BUILD", bool, default=False)
"""
``True`` value if sphinx is building, otherwise ``False``.

Set Environment Variable ``SPHINX_BUILD`` to change this value manually.
"""

FS_CACHE_PATH = get("FS_CACHE_PATH", str, default="./.fs_cache")
"""
Local Path to be used for storing cache data.
"""

FS_CACHE_RESET = get("FS_CACHE_RESET", bool, default=False)
"""
``True`` if existing caches are to be abandoned.
"""

LOGGING_LEVEL: Literal[
    "debug",
    "info",
    "warning",
    "error",
    "critical",
] = get("LOGGING_LEVEL", logging, default="debug")
"""
Logging level for runtime.

One of 5 literal :class:`str`:

- ``debug``
- ``info``
- ``warning``
- ``error``
- ``critical``

Case-insensitive.
"""
