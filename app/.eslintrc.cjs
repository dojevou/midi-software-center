/** @type {import('eslint').Linter.Config} */
module.exports = {
  root: true,
  parser: '@typescript-eslint/parser',
  parserOptions: {
    ecmaVersion: 2022,
    sourceType: 'module',
    project: './tsconfig.json',
    extraFileExtensions: ['.svelte'],
  },
  env: {
    browser: true,
    es2022: true,
    node: true,
  },
  globals: {
    NodeJS: 'readonly', // Node.js types used in setTimeout return types
  },
  plugins: [
    '@typescript-eslint',
    'svelte',
  ],
  extends: [
    'eslint:recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:@typescript-eslint/recommended-requiring-type-checking',
    'plugin:svelte/recommended',
  ],
  overrides: [
    {
      files: ['*.svelte'],
      parser: 'svelte-eslint-parser',
      parserOptions: {
        parser: '@typescript-eslint/parser',
      },
    },
  ],
  rules: {
    // TypeScript-specific rules
    '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
    '@typescript-eslint/no-explicit-any': 'warn',
    '@typescript-eslint/explicit-function-return-type': 'off',
    '@typescript-eslint/no-non-null-assertion': 'warn',
    '@typescript-eslint/prefer-nullish-coalescing': 'off', // TODO: Enable after migration
    '@typescript-eslint/prefer-optional-chain': 'warn',
    '@typescript-eslint/no-unsafe-assignment': 'off', // TODO: Enable after migration
    '@typescript-eslint/no-unsafe-member-access': 'off', // TODO: Enable after migration
    '@typescript-eslint/no-unsafe-call': 'off', // TODO: Enable after migration
    '@typescript-eslint/no-unsafe-argument': 'off', // TODO: Enable after migration
    '@typescript-eslint/no-unsafe-return': 'off', // TODO: Enable after migration
    '@typescript-eslint/require-await': 'off', // TODO: Enable after migration (async stubs)
    '@typescript-eslint/restrict-template-expressions': 'off', // TODO: Enable after migration
    '@typescript-eslint/restrict-plus-operands': 'off', // TODO: Enable after migration

    // Tauri/IPC patterns - allow floating promises for invoke
    // Use void operator: void invoke('command') for fire-and-forget
    '@typescript-eslint/no-floating-promises': ['warn', {
      ignoreVoid: true,
      ignoreIIFE: true,
    }],
    '@typescript-eslint/no-misused-promises': ['warn', {
      checksVoidReturn: false,
    }],

    // Svelte-specific rules
    'svelte/valid-compile': 'warn', // Downgrade to warn during migration (a11y issues)
    'svelte/no-at-html-tags': 'warn',
    'svelte/no-unused-svelte-ignore': 'warn',
    'svelte/require-each-key': 'error',

    // General code quality
    'no-console': 'off', // TODO: Enable after migrating to logger utility
    'no-debugger': 'warn',
    'prefer-const': 'error',
    'no-var': 'error',
    'eqeqeq': ['error', 'always'],
    'curly': ['error', 'all'],
    'no-throw-literal': 'error',
    'no-case-declarations': 'warn', // Downgrade during migration

    // Import organization
    'sort-imports': 'off', // TODO: Enable after cleaning up imports
  },
  ignorePatterns: [
    'node_modules/',
    'dist/',
    'build/',
    '.svelte-kit/',
    'src-tauri/',
    '*.config.js',
    '*.config.ts',
    'vite.config.ts',
  ],
  settings: {
    svelte: {
      compileOptions: {
        postcss: true,
      },
    },
  },
};
