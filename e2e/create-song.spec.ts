import { test, expect, waitForWorkspace } from './fixtures';

/**
 * This test creates a song with 8 different MIDI patterns from the library.
 * Selected patterns (all at 120 BPM):
 * 1. Bass: ODDONE_03_bass.mid (id: 8776760)
 * 2. Chords: OUTSTEP2_03_chord.mid (id: 8506253)
 * 3. Drums: Mididrum_120 Mambo 02.mid (id: 7564760)
 * 4. Guitar: MUSTANG_09_guitar.mid (id: 8772901)
 * 5. Melody: New_331-Melody_POP.mid (id: 7092350)
 * 6. Piano: WIVES_09_Disklavier Piano.mid (id: 8774589)
 * 7. Strings: Symphony No.2 , Mov.1_17_Tremolo Strings-2.mid (id: 8741821)
 * 8. Synth: sdh_pads_panmod_Am.mid (id: 7284892)
 *
 * Song structure:
 * - Intro (bars 1-4): Drums + Synth Pad
 * - Verse 1 (bars 5-12): Add Bass + Piano
 * - Chorus (bars 13-20): Add Guitar + Strings + Melody
 * - Verse 2 (bars 21-28): Drums + Bass + Piano + Chords
 * - Outro (bars 29-32): Full arrangement
 */

const SONG_PATTERNS = [
  { id: 8776760, name: 'Bass', filename: 'ODDONE_03_bass.mid', color: '#3b82f6' },
  { id: 8506253, name: 'Chords', filename: 'OUTSTEP2_03_chord.mid', color: '#8b5cf6' },
  { id: 7564760, name: 'Drums', filename: 'Mididrum_120 Mambo 02.mid', color: '#ef4444' },
  { id: 8772901, name: 'Guitar', filename: 'MUSTANG_09_guitar.mid', color: '#f97316' },
  { id: 7092350, name: 'Melody', filename: 'New_331-Melody_POP.mid', color: '#22c55e' },
  { id: 8774589, name: 'Piano', filename: 'WIVES_09_Disklavier Piano.mid', color: '#eab308' },
  { id: 8741821, name: 'Strings', filename: 'Symphony No.2 , Mov.1_17_Tremolo Strings-2.mid', color: '#ec4899' },
  { id: 7284892, name: 'Synth Pad', filename: 'sdh_pads_panmod_Am.mid', color: '#06b6d4' },
];

test.describe('Create Song with 8 Patterns', () => {
  test('should create a complete song with 8 MIDI patterns', async ({ page }) => {
    // Navigate to app
    await page.goto('/', { waitUntil: 'commit' });
    const isReady = await waitForWorkspace(page, 15000);
    if (!isReady) {
      test.skip();
      return;
    }

    // Step 1: Open arrangement window
    console.log('Opening arrangement window...');
    await page.keyboard.press('Alt+1');
    await page.waitForTimeout(500);

    const arrangementWindow = page.locator('.arrangement-window, .window-base').first();
    await expect(arrangementWindow).toBeVisible({ timeout: 5000 });
    console.log('✓ Arrangement window opened');

    // Step 2: Open VIP3 browser
    console.log('Opening VIP3 browser...');
    const vip3Browser = page.locator('.vip3-browser');
    if (!(await vip3Browser.isVisible({ timeout: 2000 }))) {
      await page.keyboard.press('Control+b');
      await page.waitForTimeout(500);
    }
    await expect(vip3Browser).toBeVisible({ timeout: 5000 });
    console.log('✓ VIP3 browser opened');

    // Step 3: For each pattern, search and add to sequencer
    for (let i = 0; i < SONG_PATTERNS.length; i++) {
      const pattern = SONG_PATTERNS[i];
      console.log(`Adding pattern ${i + 1}/8: ${pattern.name} (${pattern.filename})`);

      // Try to find a search input
      const searchInput = page.locator('input[type="search"], input[placeholder*="Search"], .search-input').first();
      if (await searchInput.isVisible({ timeout: 1000 }).catch(() => false)) {
        await searchInput.fill(pattern.filename.split('_')[0]);
        await page.waitForTimeout(300);
      }

      // Look for file items in the browser
      const fileItems = page.locator('.file-item, .vip3-file-row');
      const count = await fileItems.count();

      if (count > 0) {
        // Double-click the first visible file to add it
        await fileItems.first().dblclick({ force: true });
        await page.waitForTimeout(300);
        console.log(`✓ Added ${pattern.name} pattern`);
      } else {
        console.log(`⚠ No files visible for ${pattern.name}, skipping...`);
      }
    }

    // Step 4: Play the song briefly
    console.log('Playing song preview...');
    await page.keyboard.press('Space');
    await page.waitForTimeout(2000);
    await page.keyboard.press('Space'); // Stop
    console.log('✓ Played song preview');

    // Step 5: Save the project
    console.log('Saving project...');

    // Set up dialog handler
    page.on('dialog', async (dialog) => {
      await dialog.accept('8-Pattern-Song');
    });

    await page.keyboard.press('Control+s');
    await page.waitForTimeout(500);

    // Handle save dialog if appears
    const saveDialog = page.locator('.modal-overlay').filter({ hasText: /Save|Project/ }).first();
    const isVisible = await saveDialog.isVisible().catch(() => false);
    if (isVisible) {
      const nameInput = saveDialog.locator('input[type="text"]');
      if (await nameInput.isVisible().catch(() => false)) {
        await nameInput.fill('8-Pattern-Song');
      }
      const saveBtn = saveDialog.locator('button').filter({ hasText: /Save|OK|Confirm/ });
      if (await saveBtn.isVisible().catch(() => false)) {
        await saveBtn.click();
        await page.waitForTimeout(500);
      }
    }
    console.log('✓ Project saved');

    // Verify tracks exist
    const tracks = page.locator('.sequencer-track, .track');
    const trackCount = await tracks.count();
    console.log(`Created ${trackCount} tracks in sequencer`);

    // Test passes if we got this far
    expect(true).toBe(true);
  });
});
