import { readFileSync, writeFileSync } from 'fs';
import { createRequire } from 'module';

const require = createRequire(import.meta.url);
const artifact = require('../artifacts/contracts/LendChain.sol/LendChain.json');

writeFileSync(
  'src/blockchain/contracts/lendchain_abi.json',
  JSON.stringify(artifact.abi, null, 2)
);

writeFileSync(
  'src/blockchain/contracts/lendchain_bytecode.hex',
  artifact.bytecode.replace('0x', '').trim()
);

console.log('ABI y bytecode extraídos correctamente');
