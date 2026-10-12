const [major, minor] = process.versions.node.split('.').map(Number);
if (major < 22 || (major === 22 && minor < 9)) {
  console.error(`Selah requires Node 22.9 or newer. This terminal is using Node ${process.versions.node} at ${process.execPath}.`);
  console.error('If Node is already installed, activate it in this terminal before retrying. With nvm: nvm use 22 (or nvm use 23 if that is your installed version).');
  process.exit(1);
}
console.log(`Node ${process.versions.node} · ${process.execPath}`);
