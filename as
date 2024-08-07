#! /usr/bin/env python3

import sys, os
from pathlib import Path
import subprocess as sp

prev_cwd = Path.cwd();
script_dir = Path(__file__).resolve().parent
os.chdir(script_dir)

sp.check_call(["cargo", "build", "--bin", "as-cli"])
os.chdir(prev_cwd)

bin_path = script_dir / "target/debug/as-cli"
sp.check_call([bin_path, *sys.argv[1:]])

