// Fixture: mixes declared, stdlib, and ghost imports.
//   express              -> declared in package.json
//   node:fs              -> Node.js stdlib (ignored by scanner)
//   axios                -> ghost (require)
//   lodash               -> ghost (ESM default import)
//   @scope/zlib-tools    -> ghost (dynamic import, scoped)
const express = require("express");
const axios = require("axios");
const fs = require("node:fs");

import lodash from 'lodash';

async function load() {
  const tools = await import('@scope/zlib-tools');
  return tools;
}

module.exports = { express, axios, fs, lodash, load };
