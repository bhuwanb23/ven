"""Fixture: mixes declared, stdlib, and ghost imports.

  requests   -> declared in requirements.txt (not a ghost)
  os, sys    -> stdlib (ignored by scanner)
  flask      -> ghost (plain name)
  yaml       -> ghost, renamed to pyyaml by the scanner's rename table
  sklearn    -> ghost, renamed to scikit-learn by the scanner's rename table
"""

import os
import sys

import requests
import flask
import yaml
import sklearn

from os import path  # stdlib submodule, must NOT be flagged


def main() -> None:
    _ = (os, sys, path, requests, flask, yaml, sklearn)
