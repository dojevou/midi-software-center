import fs from 'fs';
import path from 'path';
import { parse } from 'acorn';
import { simple } from 'acorn-walk';

const projectRoot = path.resolve('.');
const componentFiles = [
  'app/src/lib/windows/MixerWindow.svelte',
  'app/src/lib/windows/DAWWindow.svelte',
  'app/src/lib/windows/DatabaseWindow.svelte',
  'app/src/lib/windows/PipelineWindow.svelte',
  'app/src/lib/components/StatusBar.svelte'
];

function findFile(pathToCheck: string): boolean {
  try {
    fs.accessSync(pathToCheck, fs.constants.F_OK);
    return true;
  } catch {
    return false;
  }
}

function auditFile(filePath: string) {
  const code = fs.readFileSync(filePath, 'utf8');
  const ast = parse(code, { 
    ecmaVersion: 2020, 
    sourceType: 'module',
    onInsertedSemicolon: true 
  });

  const missingImports = [];

  simple(ast, {
    ImportDeclaration(node) {
      if (node.source && node.source.value) {
        const importPath = node.source.value;
        // Check if it's a relative import or absolute
        if (importPath.startsWith('$lib/')) {
          const fullPath = path.join(projectRoot, importPath.replace('$lib', 'src/lib'));
          if (!findFile(fullPath + '.ts') && !findFile(fullPath + '.js') && !findFile(fullPath + '.svelte')) {
            missingImports.push({
              type: 'module',
              name: importPath,
              node
            });
          }
        } else if (importPath.startsWith('@tauri-apps/')) {
          // Check if package.json has it
          const packageJson = JSON.parse(fs.readFileSync(path.join(projectRoot, 'app/package.json'), 'utf8'));
          if (!packageJson.dependencies[importPath] && !packageJson.devDependencies[importPath]) {
            missingImports.push({
              type: 'package',
              name: importPath,
              node
            });
          }
        }
      }
    },
    ImportSpecifier(node) {
      // Check if the imported name is exported from the module
      // This is more complex, skip for now or implement simple check
    }
  });

  if (missingImports.length > 0) {
    console.log(`\nFile: ${filePath}`);
    missingImports.forEach(missing => {
      console.log(`Missing: ${missing.type} - ${missing.name}`);
    });
  }
}

componentFiles.forEach(auditFile);

console.log('\nAudit complete. Check the output above for missing dependencies.');