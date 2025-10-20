const fs = require('fs');
const path = require('path');

// Load the IDL
const idlPath = path.join(__dirname, 'target/idl/arcium_poker.json');
const idl = JSON.parse(fs.readFileSync(idlPath, 'utf8'));

console.log('Original accounts:', JSON.stringify(idl.accounts, null, 2));

// Fix the accounts section by adding type references
if (idl.accounts && idl.types) {
  idl.accounts = idl.accounts.map(account => {
    // Find the matching type definition
    const typeDefinition = idl.types.find(t => t.name === account.name);
    
    if (typeDefinition) {
      return {
        ...account,
        type: typeDefinition.type
      };
    }
    return account;
  });
}

console.log('\nFixed accounts:', JSON.stringify(idl.accounts, null, 2).substring(0, 500));

// Save the fixed IDL
fs.writeFileSync(idlPath, JSON.stringify(idl, null, 2));
console.log('\n✅ IDL fixed and saved!');

// Also copy to frontend
const frontendPath = path.join(__dirname, '../arcium-poker-frontend/src/arcium_poker.json');
fs.writeFileSync(frontendPath, JSON.stringify(idl, null, 2));
console.log('✅ Copied to frontend!');
