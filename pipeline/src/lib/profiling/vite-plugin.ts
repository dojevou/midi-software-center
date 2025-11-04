/**
 * Phase 7C Frontend Optimization - Vite Plugin for Bundle Analysis
 *
 * Vite plugin to automatically analyze bundle size and generate reports
 *
 * Usage in vite.config.ts:
 * ```typescript
 * import { bundleAnalyzerPlugin } from './src/lib/profiling/vite-plugin';
 *
 * export default defineConfig({
 *   plugins: [
 *     sveltekit(),
 *     bundleAnalyzerPlugin()
 *   ]
 * });
 * ```
 *
 * @module profiling/vite-plugin
 */

import type { Plugin, ResolvedConfig } from 'vite';
import fs from 'fs';
import path from 'path';

interface BundleAnalyzerOptions {
  /**
   * Output directory for reports
   */
  outDir?: string;

  /**
   * Enable detailed logging
   */
  verbose?: boolean;

  /**
   * Maximum bundle size threshold (bytes)
   */
  maxBundleSize?: number;

  /**
   * Maximum chunk size threshold (bytes)
   */
  maxChunkSize?: number;

  /**
   * Whether to fail build on size threshold exceeded
   */
  failOnThreshold?: boolean;
}

export function bundleAnalyzerPlugin(options: BundleAnalyzerOptions = {}): Plugin {
  const {
    outDir = '.bundle-analysis',
    verbose = false,
    maxBundleSize = 500 * 1024, // 500KB
    maxChunkSize = 200 * 1024, // 200KB
    failOnThreshold = false
  } = options;

  let config: ResolvedConfig;

  return {
    name: 'bundle-analyzer',
    apply: 'build',

    configResolved(resolvedConfig) {
      config = resolvedConfig;
    },

    async closeBundle() {
      const buildDir = config.build.outDir;
      const manifest = await loadManifest(buildDir);

      if (!manifest) {
        if (verbose) {
          console.log('[bundle-analyzer] No manifest found, skipping analysis');
        }
        return;
      }

      const analysis = analyzeBundle(manifest, buildDir);
      await generateReport(analysis, outDir, verbose);

      // Check thresholds
      const violations = checkThresholds(analysis, maxBundleSize, maxChunkSize);
      if (violations.length > 0) {
        violations.forEach(v => console.warn(`[bundle-analyzer] ${v}`));

        if (failOnThreshold) {
          throw new Error('Bundle size threshold exceeded');
        }
      }
    }
  };
}

/**
 * Load Vite manifest
 */
async function loadManifest(buildDir: string): Promise<Record<string, any> | null> {
  const manifestPath = path.join(buildDir, '.vite/manifest.json');

  try {
    const content = fs.readFileSync(manifestPath, 'utf-8');
    return JSON.parse(content);
  } catch {
    return null;
  }
}

/**
 * Analyze bundle structure
 */
function analyzeBundle(manifest: Record<string, any>, buildDir: string) {
  const chunks: Array<{ name: string; size: number; type: string }> = [];
  let totalSize = 0;
  let jsSize = 0;
  let cssSize = 0;

  for (const [file, info] of Object.entries(manifest)) {
    const filePath = path.join(buildDir, info.file || file);

    try {
      const stats = fs.statSync(filePath);
      const size = stats.size;

      totalSize += size;

      let type = 'asset';
      if (file.endsWith('.js')) {
        type = 'js';
        jsSize += size;
      } else if (file.endsWith('.css')) {
        type = 'css';
        cssSize += size;
      }

      chunks.push({
        name: file,
        size,
        type
      });
    } catch {
      // File doesn't exist, skip
    }
  }

  // Sort by size
  chunks.sort((a, b) => b.size - a.size);

  return {
    totalSize,
    jsSize,
    cssSize,
    assetSize: totalSize - jsSize - cssSize,
    chunks,
    chunkCount: chunks.filter(c => c.type === 'js').length
  };
}

/**
 * Generate analysis report
 */
async function generateReport(analysis: any, outDir: string, verbose: boolean) {
  // Create output directory
  if (!fs.existsSync(outDir)) {
    fs.mkdirSync(outDir, { recursive: true });
  }

  // Generate JSON report
  const jsonReport = {
    timestamp: new Date().toISOString(),
    summary: {
      totalSize: analysis.totalSize,
      jsSize: analysis.jsSize,
      cssSize: analysis.cssSize,
      assetSize: analysis.assetSize,
      chunkCount: analysis.chunkCount
    },
    chunks: analysis.chunks
  };

  fs.writeFileSync(
    path.join(outDir, 'bundle-report.json'),
    JSON.stringify(jsonReport, null, 2)
  );

  // Generate markdown report
  const mdReport = generateMarkdownReport(analysis);
  fs.writeFileSync(path.join(outDir, 'bundle-report.md'), mdReport);

  // Console output
  console.log('\n[bundle-analyzer] Bundle Analysis');
  console.log(`Total Size: ${formatBytes(analysis.totalSize)}`);
  console.log(`  JS: ${formatBytes(analysis.jsSize)} (${((analysis.jsSize / analysis.totalSize) * 100).toFixed(1)}%)`);
  console.log(`  CSS: ${formatBytes(analysis.cssSize)} (${((analysis.cssSize / analysis.totalSize) * 100).toFixed(1)}%)`);
  console.log(`  Assets: ${formatBytes(analysis.assetSize)} (${((analysis.assetSize / analysis.totalSize) * 100).toFixed(1)}%)`);
  console.log(`Chunks: ${analysis.chunkCount}`);

  if (verbose) {
    console.log('\nTop 5 Largest Chunks:');
    analysis.chunks.slice(0, 5).forEach((chunk: any, i: number) => {
      console.log(`  ${i + 1}. ${chunk.name}: ${formatBytes(chunk.size)}`);
    });
  }

  console.log(`\nDetailed report: ${path.join(outDir, 'bundle-report.md')}`);
}

/**
 * Generate markdown report
 */
function generateMarkdownReport(analysis: any): string {
  let md = '# Bundle Analysis Report\n\n';
  md += `Generated: ${new Date().toISOString()}\n\n`;

  md += '## Summary\n\n';
  md += `- **Total Size:** ${formatBytes(analysis.totalSize)}\n`;
  md += `- **JavaScript:** ${formatBytes(analysis.jsSize)} (${((analysis.jsSize / analysis.totalSize) * 100).toFixed(1)}%)\n`;
  md += `- **CSS:** ${formatBytes(analysis.cssSize)} (${((analysis.cssSize / analysis.totalSize) * 100).toFixed(1)}%)\n`;
  md += `- **Assets:** ${formatBytes(analysis.assetSize)} (${((analysis.assetSize / analysis.totalSize) * 100).toFixed(1)}%)\n`;
  md += `- **Chunk Count:** ${analysis.chunkCount}\n\n`;

  md += '## Top 10 Largest Chunks\n\n';
  md += '| Rank | File | Size | Type |\n';
  md += '|------|------|------|------|\n';

  analysis.chunks.slice(0, 10).forEach((chunk: any, i: number) => {
    md += `| ${i + 1} | ${chunk.name} | ${formatBytes(chunk.size)} | ${chunk.type} |\n`;
  });

  md += '\n## Optimization Recommendations\n\n';

  if (analysis.totalSize > 500 * 1024) {
    md += '- ⚠️ **Total bundle size exceeds 500KB** - Consider code splitting and lazy loading\n';
  }

  const largeChunks = analysis.chunks.filter((c: any) => c.size > 200 * 1024);
  if (largeChunks.length > 0) {
    md += `- ⚠️ **${largeChunks.length} chunks exceed 200KB** - Split large modules\n`;
  }

  if (analysis.jsSize / analysis.totalSize > 0.8) {
    md += '- ℹ️ **High JS to CSS ratio** - Consider extracting CSS\n';
  }

  md += '\n## All Chunks\n\n';
  md += '| File | Size | Type |\n';
  md += '|------|------|------|\n';

  analysis.chunks.forEach((chunk: any) => {
    md += `| ${chunk.name} | ${formatBytes(chunk.size)} | ${chunk.type} |\n`;
  });

  return md;
}

/**
 * Check size thresholds
 */
function checkThresholds(
  analysis: any,
  maxBundleSize: number,
  maxChunkSize: number
): string[] {
  const violations: string[] = [];

  if (analysis.totalSize > maxBundleSize) {
    violations.push(
      `Total bundle size (${formatBytes(analysis.totalSize)}) exceeds limit (${formatBytes(maxBundleSize)})`
    );
  }

  const oversizedChunks = analysis.chunks.filter((c: any) => c.size > maxChunkSize && c.type === 'js');
  if (oversizedChunks.length > 0) {
    oversizedChunks.forEach((chunk: any) => {
      violations.push(
        `Chunk "${chunk.name}" (${formatBytes(chunk.size)}) exceeds limit (${formatBytes(maxChunkSize)})`
      );
    });
  }

  return violations;
}

/**
 * Format bytes to human-readable string
 */
function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(2)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}
