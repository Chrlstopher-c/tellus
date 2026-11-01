import js from '@eslint/js';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  {
    ignores: ['out/**', 'dist/**', 'node_modules/**', 'logs/**', 'coverage/**', 'mockup/**', 'simulation/**', 'docs/**'],
  },
  js.configs.recommended,
  tseslint.configs.recommended,
  {
    files: ['**/*.{ts,tsx,mts}'],
    rules: {
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/consistent-type-assertions': ['error', { assertionStyle: 'never' }],
      '@typescript-eslint/explicit-module-boundary-types': 'error',
      'max-lines': ['error', { max: 500 }],
      'max-lines-per-function': ['error', { max: 35 }],
      'max-len': ['error', { code: 120 }],
      'no-console': 'error',
    },
  },
);
