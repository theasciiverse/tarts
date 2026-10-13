#!/usr/bin/env python3
"""Records asciiverse/tarts.cast, the catalog preview, at 100x30: `tarts tour`
with a few seconds of each effect, moved on with n. Needs asciinema.
Usage: record-preview.py <tarts binary> <out.cast>
Then asciiverse/cast-cap-fps.py out.cast asciiverse/tarts.cast 25: tarts
draws at 60 fps, and plasma repaints every cell each frame (58 MB raw)."""
import fcntl, os, pty, select, shutil, struct, sys, tempfile, termios, time

binary, out = os.path.abspath(sys.argv[1]), os.path.abspath(sys.argv[2])
home = tempfile.mkdtemp()  # no ~/.config/tarts.toml: the defaults
env = dict(os.environ, HOME=home, TERM="xterm-256color", COLORTERM="truecolor", LANG="C.UTF-8")
pid, fd = pty.fork()
if pid == 0:
    fcntl.ioctl(0, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))
    os.execvpe("asciinema", ["asciinema", "rec", "--overwrite", "-q",
        "--capture-env", "TERM,COLORTERM,LANG", "-c", f"{binary} tour", out], env)
fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", 30, 100, 0, 0))

def pump(t):
    end = time.time() + t
    while time.time() < end:
        if select.select([fd], [], [], 0.05)[0]:
            try:
                os.read(fd, 1 << 16)
            except OSError:
                return

# the tour runs matrix, donut, boids, plasma, fire, pipes, life, cube,
# constellation, crab, ...; seconds on each, 0 skips one
for seconds in [3.6, 3.3, 3.3, 3.0, 3.3, 3.6, 3.3, 3.0, 0, 3.6]:
    pump(seconds)
    os.write(fd, b"n")
    pump(0.05)
os.write(fd, b"q")
for _ in range(50):
    if os.waitpid(pid, os.WNOHANG)[0]:
        break
    time.sleep(0.1)
else:
    os.kill(pid, 15)
shutil.rmtree(home, ignore_errors=True)
