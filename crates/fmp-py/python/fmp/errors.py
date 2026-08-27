"""Runtime source shim for the native :mod:`fmp.errors` module.

The compiled exception hierarchy is registered while :mod:`fmp._native`
initializes, so this file is documentation-only during normal imports. The
matching ``errors.pyi`` file is the hand-maintained typing contract.
"""
