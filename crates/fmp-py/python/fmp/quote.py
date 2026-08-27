"""Runtime source shim for the native ``fmp.quote`` submodule.

The real ``fmp.quote`` is a compiled submodule registered in ``sys.modules``
while ``fmp._native`` initializes, so this file does not run during normal
imports. It exists so source-aware tools can discover the module and resolve
its hand-maintained ``quote.pyi`` typing contract.
"""
