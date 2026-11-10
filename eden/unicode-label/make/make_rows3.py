#!/usr/bin/env python3
"""Rows for the 0.10.4 (paged codepoint index) boots, built with make_rows2's helpers from eden/rows-dun:

  eden/rows-dun4  label pair: a 0.10.3 (bc22c80, the card's build), b 0.10.4 (21c0009); both labels draw their own
                  glyphs; the font lines in and after the match (b's index paged, a's dense); one match to sync_end
  eden/rows-dun5  name change: a 0.10.4, b the 0.10.4 diag build with the label feed; per name the glyphs, the
                  dynamic set, the index mode and bytes, and the index-bytes trace; an evidence GIF of b's label
                  changing from the Japanese to the Korean name; a's label; one match to sync_end
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import make_rows2 as base  # noqa: E402

D = base.D
LABEL = base.LABEL
FEED = base.FEED
both = base.both


def main():
    base.build(
        'rows-dun4',
        'Deluxe 0.10.4 (paged codepoint index), the label pair: a runs 0.10.3 (bc22c80), b 0.10.4 (21c0009); both '
        'profiles are named "Æsir テスト"; detector and tile cursor on both.',
        base.common() + [
            {'id': 'm1-label', 'start': 'b-searching', 'end': 'match-over', 'budget_min': 12,
             'evidence': ['a (0.10.3) and b (0.10.4): Æ テ ス ト draw their own glyphs',
                          'b: imgui_font_index=paged, the dynamic set テ ス ト; a: the dense index of 0.10.3',
                          'both detectors: sync_end desync_seen=0'],
             'steps': [
                 *base.a_to_session(both),
                 {'run': D + 'label_glyphs', 'on': 'b', 'check': True, 'args': {'expect': 'own', 'active': True}},
                 {'run': D + 'label_glyphs', 'on': 'a', 'check': True, 'args': {'expect': 'own', 'active': True}},
                 {'run': D + 'font_sample', 'on': 'b', 'check': True,
                  'args': {'tag': 'in-match', 'contains': LABEL, 'index_mode': 'paged'}},
                 {'run': D + 'font_sample', 'on': 'a', 'check': True, 'args': {'tag': 'in-match', 'contains': LABEL}},
                 *base.match_end()]},
            {'id': 'f1-fonts', 'start': 'match-over', 'end': 'match-over', 'budget_min': 1, 'steps': [
                {'run': D + 'font_sample', 'on': 'b', 'check': True,
                 'args': {'tag': 'after-match', 'active': False, 'index_mode': 'paged'}},
                {'run': D + 'font_sample', 'on': 'a', 'check': True, 'args': {'tag': 'after-match', 'active': False}}]},
            *base.zrows()],
        {'state': 'Deluxe windows open in the match', 'via': 'segment', 'ref': 'segment open-deluxe (L+R+Down)'})
    feed_args = {'own': LABEL, 'names': FEED, 'index_mode': 'paged'}
    base.build(
        'rows-dun5',
        'Deluxe 0.10.4 (paged codepoint index), a name change: a runs 0.10.4 (21c0009), b the 0.10.4 diag build '
        '(21c0009 --features diag) whose diag.txt label feed shows ' + ', '.join(FEED) + ' in turn, 20 s each; both '
        'profiles are named "Æsir テスト"; detector and tile cursor on both.',
        base.common() + [
            {'id': 'm1-feed', 'start': 'b-searching', 'end': 'match-over', 'budget_min': 14,
             'evidence': ['b: for each fed name, its characters draw their own glyphs, the dynamic set is that name '
                          '(the previous one dropped, a rebuild counted), the index is paged; index-bytes trace',
                          'b: evidence GIF label-ja-ko (640 wide): the label changes from 日本語のなまえ to 강철무사',
                          'a (0.10.4): Æ テ ス ト draw their own glyphs, index paged',
                          'both detectors: sync_end desync_seen=0'],
             'steps': [
                 *base.a_to_session(('a',)),
                 {'run': D + 'follow_feed', 'on': 'b', 'check': True, 'args': {**feed_args, 'until_phase': 1}},
                 {'run': D + 'feed_wait', 'on': 'b', 'check': True, 'args': {'phase': 1, 'after_s': 15.5}},
                 {'on': 'b', 'gif': {'name': 'label-ja-ko', 'preset': 'evidence', 'fps': 5, 'length': 12, 'steps': [
                     {'run': D + 'feed_switch', 'on': 'b', 'check': True, 'args': {'index': 2, 'hold_s': 4.5}}]}},
                 {'run': D + 'follow_feed', 'on': 'b', 'check': True, 'args': feed_args},
                 {'run': D + 'label_glyphs', 'on': 'a', 'check': True, 'args': {'expect': 'own', 'active': True}},
                 {'run': D + 'font_sample', 'on': 'a', 'check': True,
                  'args': {'tag': 'in-match', 'contains': LABEL, 'index_mode': 'paged'}},
                 *base.match_end()]},
            {'id': 'f1-fonts', 'start': 'match-over', 'end': 'match-over', 'budget_min': 1, 'steps': [
                {'run': D + 'font_sample', 'on': k, 'check': True,
                 'args': {'tag': 'after-match', 'active': False, 'index_mode': 'paged'}} for k in both]},
            *base.zrows()],
        {'state': 'Deluxe windows open in the match; b label feed from diag.txt', 'via': 'fixture',
         'ref': 'segment open-deluxe; instances[b].files diag.txt'})


if __name__ == '__main__':
    sys.exit(main())
