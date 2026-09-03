import os
import signal
import time

from test_client import fixture_server


def test_client_initialized_in_parent_remains_usable_after_fork():
    if not hasattr(os, "fork"):
        return

    from fmp import FmpClient

    with fixture_server() as (base_url, _requests):
        client = FmpClient(base_url=base_url, path_prefix="", auth_mode="none")
        assert client.quote_short("AAPL")[0].symbol == "AAPL"

        read_fd, write_fd = os.pipe()
        child_pid = os.fork()
        if child_pid == 0:
            os.close(read_fd)
            try:
                rows = client.quote_short("AAPL")
                payload = rows[0].symbol.encode()
                exit_code = 0
            except BaseException as error:
                payload = f"{type(error).__name__}: {error}".encode()
                exit_code = 1
            os.write(write_fd, payload)
            os.close(write_fd)
            os._exit(exit_code)

        os.close(write_fd)
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            waited_pid, status = os.waitpid(child_pid, os.WNOHANG)
            if waited_pid == child_pid:
                break
            time.sleep(0.05)
        else:
            os.kill(child_pid, signal.SIGKILL)
            os.waitpid(child_pid, 0)
            raise AssertionError("the post-fork request did not complete within 10 seconds")

        payload = os.read(read_fd, 65_536)
        os.close(read_fd)
        assert os.WIFEXITED(status), f"child terminated abnormally with status {status}"
        assert os.WEXITSTATUS(status) == 0, payload.decode(errors="replace")
        assert payload == b"AAPL"
