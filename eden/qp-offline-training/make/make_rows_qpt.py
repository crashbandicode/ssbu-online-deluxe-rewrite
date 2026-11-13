"""Write rows.json for the qp-fix Eden boot (qpt-*): a LAN Quickplay pair with the GDB stub on.

a: Games & More -> Training on a fresh boot (offline Training must read offline/training).
b: Online -> Quickplay -> READY -> the waiting room (quickplay/training), then Leave Battle, the main menu and
   Games & More -> Training (the kept Quickplay becomes Offline at the Training stage, event 107 a=5).
Every gate after the menus reads Deluxe's own files (qp_steps.py); library segments and refs are pinned by sha256.
"""
import hashlib
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parents[1]
LIB = Path('/home/intpa/ssbu-mod-playbook/tools/eden-runner/library')


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def local_seg(name, cap):
    path = HERE / 'segments' / (name + '.edenmacro.json')
    return {'file': 'segments/%s.edenmacro.json' % name, 'sha256': sha(path), 'cap': cap}


def lib_seg(name):
    """A shared-library segment, copied into segments/ (the run doesn't depend on the host clone's library state)."""
    index = json.loads((LIB / 'index.json').read_text())['segments'][name]
    local = HERE / 'segments' / (name + '.edenmacro.json')
    assert sha(local) == sha(LIB / index['file']) == index['sha256'], name
    # No "library" key: the runner would merge the library entry's start gate (enter-training's "ready" is not a
    # gate here; qpt-y1's first try stopped on that KeyError). The hooks check each screen themselves.
    return {'file': 'segments/%s.edenmacro.json' % name, 'sha256': index['sha256'], 'cap': index.get('cap', 60),
            'from_library': name}


def local_ref(name):
    return {'reference': 'refs/' + name, 'sha256': sha(HERE / 'refs' / name)}


def lib_ref(name):
    """A shared-library reference, copied into refs/."""
    gate = json.loads((LIB / 'index.json').read_text())['gates'][name]
    local = HERE / 'refs' / Path(gate['reference']).name
    assert sha(local) == sha(LIB / gate['reference']) == gate['sha256'], name
    return {'reference': 'refs/' + local.name, 'sha256': gate['sha256']}


FRAME = [1280, 674]
GATES = {
    'loaded': {'library': 'loaded'},
    'installed': {'kind': 'log', 'source': 'detector_log', 'whole_file': True, 'cap': 120,
                  'pattern': 'plugin=ssbu-desync-detector/\\S+ [^\\n]*heap_allocations=0'},
    'deluxe-embedded': {'kind': 'log', 'source': 'deluxe_install', 'whole_file': True, 'cap': 120,
                        'pattern': 'overlay_host_ready'},
    'deluxe-build': {'kind': 'log', 'source': 'deluxe_events', 'whole_file': True, 'cap': 60,
                     'pattern': 'Deluxe rewrite 0\\.10\\.5-1173bead82fa-'},
    'main': dict(kind='image', roi=[150, 265, 450, 335], tolerance=8, cap=45, reference_is_roi=True, frame=FRAME,
                 scale='fit', recover='attract-cancel', recover_times=4, **local_ref('main.png')),
    'stage': dict(kind='image', roi=[110, 445, 290, 495], tolerance=8, cap=45, reference_is_roi=True,
                  frame=[1280, 720], scale='fit', **lib_ref('stage')),
    'css': dict(kind='image', roi=[55, 55, 675, 155], tolerance=8, cap=60, reference_is_roi=True,
                frame=[1280, 720], scale='fit', **lib_ref('css')),
    'online-menu': dict(kind='image', roi=[40, 5, 300, 45], tolerance=8, cap=40, reference_is_roi=True, frame=FRAME,
                        scale='fit', **local_ref('online-menu-roi.png')),
    'quickplay-menu': dict(kind='image', roi=[40, 5, 300, 45], tolerance=8, cap=40, reference_is_roi=True,
                           frame=FRAME, scale='fit', **local_ref('quickplay-menu-roi.png')),
    'css-tile': {'kind': 'log', 'source': 'tile_log', 'pattern': 'settled screen=css at=(\\d+) hovered=\\1\\b',
                 'cap': 40},
    'ready-banner-any': dict(kind='image', roi=[200, 110, 600, 240], tolerance=40, cap=30, reference_is_roi=True,
                             frame=FRAME, scale='fit', **local_ref('ready-banner-roi.png')),
}
SEGMENTS = {
    'to-online': local_seg('to-online', 60),
    'to-quickplay': local_seg('to-quickplay', 40),
    'to-css': local_seg('to-css', 60),
    'tile-up': local_seg('tile-up', 20),
    'pick': local_seg('pick', 20),
    'attract-cancel': lib_seg('attract-cancel'),
    'main-stage': lib_seg('main-stage'),
    'stage-css': lib_seg('stage-css'),
    'enter-training': lib_seg('enter-training'),
    'quit-focus': local_seg('quit-focus', 20),
    'quit-confirm': local_seg('quit-confirm', 20),
}


def both(step):
    return [dict(step, on='a'), dict(step, on='b')]


def one(step):
    return [dict(step, on='a')]


# One instance (a free slot, not two): the same rows in a chain on a. A first (a fresh boot, no online session yet),
# then out of Training to the main menu, the Quickplay search, and Training after it.
SINGLE = [
    {'id': 'q00-load', 'start': 'frontend-ready', 'end': 'loaded', 'on_failure': 'stop', 'budget_min': 5,
     'steps': one({'run': 'gdb_rc11.py:gdb_on', 'check': True})
     + one({'call': 'game.load', 'params': {'path': '@game'}, 'job_cap': 240})
     + one({'run': 'gdb_rc11.py:gdb_kick', 'check': True})
     + one({'gate': 'loaded'}) + one({'gate': 'installed'}) + one({'gate': 'deluxe-embedded'})
     + one({'gate': 'deluxe-build'})
     + [{'reason': 'other plugins finish booting', 'wait_ms': 15000}] + one({'shot': 'boot'})},
    {'id': 'q01-main', 'on': 'a', 'start': 'loaded', 'end': 'main', 'on_failure': 'stop', 'budget_min': 2,
     'steps': one({'run': 'qp_steps.py:to_main', 'check': True}) + one({'gate': 'main'})},
    {'id': 'rA-offline-training', 'on': 'a', 'start': 'main', 'end': 'a-training', 'on_failure': 'stop',
     'budget_min': 5, 'steps': [{'run': 'qp_steps.py:offline_training', 'on': 'a', 'check': True}]},
    {'id': 'rX-exit-training', 'on': 'a', 'start': 'a-training', 'end': 'main', 'on_failure': 'stop',
     'budget_min': 3, 'steps': [{'run': 'qp_steps.py:exit_training', 'on': 'a', 'check': True}]},
    {'id': 'rB-waiting-room', 'on': 'a', 'start': 'main', 'end': 'a-waiting', 'on_failure': 'continue',
     'budget_min': 5, 'steps': [{'run': 'qp_steps.py:qp_waiting', 'on': 'a', 'check': True}]},
    {'id': 'rD-training-after-qp', 'on': 'a', 'start': 'a-waiting', 'start_gate': 'loaded', 'recover': [],
     'end': 'a-training', 'on_failure': 'continue', 'budget_min': 6,
     'steps': [{'run': 'qp_steps.py:training_after_qp', 'on': 'a', 'check': True}]},
    {'id': 'z-a', 'on': 'a', 'start': 'a-training', 'start_gate': 'loaded', 'recover': [], 'end': 'a-training',
     'budget_min': 1,
     'steps': [{'run': 'gdb_rc11.py:z_stub', 'on': 'a', 'check': True},
               {'run': 'qp_steps.py:final_evidence', 'on': 'a', 'check': True}]},
]

ROWS = [
    {'id': 'q00-load', 'start': 'frontend-ready', 'end': 'loaded', 'on_failure': 'stop', 'budget_min': 6,
     'steps': both({'run': 'gdb_rc11.py:gdb_on', 'check': True})
     + both({'call': 'game.load', 'params': {'path': '@game'}, 'job_cap': 240})
     + both({'run': 'gdb_rc11.py:gdb_kick', 'check': True})
     + both({'gate': 'loaded'}) + both({'gate': 'installed'}) + both({'gate': 'deluxe-embedded'})
     + both({'gate': 'deluxe-build'})
     + [{'reason': 'other plugins finish booting', 'wait_ms': 15000}] + both({'shot': 'boot'})},
    {'id': 'q01-main', 'start': 'loaded', 'end': 'main', 'on_failure': 'stop', 'budget_min': 2,
     'steps': both({'run': 'qp_steps.py:to_main', 'check': True}) + both({'gate': 'main'})},
    # The pair's state labels chain on the instance the next row drives: after rA, b is still on the main menu.
    {'id': 'rA-offline-training', 'on': 'a', 'start': 'main', 'end': 'main', 'on_failure': 'continue',
     'budget_min': 5, 'steps': [{'run': 'qp_steps.py:offline_training', 'on': 'a', 'check': True}]},
    {'id': 'rB-waiting-room', 'on': 'b', 'start': 'main', 'recover': [], 'end': 'b-waiting',
     'on_failure': 'continue', 'budget_min': 5,
     'steps': [{'run': 'qp_steps.py:qp_waiting', 'on': 'b', 'check': True}]},
    # start_gate "loaded" only lets the runner continue after a failed rB; the hook itself refuses at once when b is
    # not in the waiting room.
    {'id': 'rD-training-after-qp', 'on': 'b', 'start': 'b-waiting', 'start_gate': 'loaded', 'recover': [],
     'end': 'b-training', 'on_failure': 'continue', 'budget_min': 6,
     'steps': [{'run': 'qp_steps.py:training_after_qp', 'on': 'b', 'check': True}]},
    {'id': 'z-a', 'on': 'a', 'start': 'b-training', 'start_gate': 'loaded', 'recover': [], 'end': 'b-training',
     'on_failure': 'continue', 'budget_min': 1,
     'steps': [{'run': 'gdb_rc11.py:z_stub', 'on': 'a', 'check': True},
               {'run': 'qp_steps.py:final_evidence', 'on': 'a', 'check': True}]},
    {'id': 'z-b', 'on': 'b', 'start': 'b-training', 'start_gate': 'loaded', 'recover': [], 'end': 'b-training',
     'budget_min': 1,
     'steps': [{'run': 'gdb_rc11.py:z_stub', 'on': 'b', 'check': True},
               {'run': 'qp_steps.py:final_evidence', 'on': 'b', 'check': True}]},
]
SETUP = [
    {'state': 'two Eden frontends up, games not loaded, the GDB stub set on (defer_load)', 'via': 'hook',
     'ref': 'gdb_rc11.py gdb_on with gdb-port-a.txt, gdb-port-b.txt and gdb-nohold.txt'},
    {'state': 'both at the main menu', 'via': 'hook', 'ref': 'qp_steps.py to_main; main gate recover attract-cancel'},
    {'state': 'a: offline Training through Games & More, P1 on tile 0', 'via': 'segment',
     'ref': 'library main-stage, stage-css, enter-training; qp_steps.py _css_to_tile0'},
    {'state': 'b: searching in the Quickplay waiting room', 'via': 'segment',
     'ref': 'segments to-online, to-quickplay, to-css, tile-up, pick (eden/unicode-label); qp_steps.py qp_waiting'},
    {'state': 'b: Leave Battle, main menu, Training', 'via': 'hook', 'ref': 'qp_steps.py training_after_qp'},
]
PLAN = {
    'about': 'Deluxe qp-fix 1173bea (NRO 2278c03d, build 0.10.5-1173bead82fa-1794575569) on both instances of a LAN '
             'Quickplay pair, the detector and the tile cursor on both, the GDB stub on: offline Training through '
             'Games & More (a), the Quickplay waiting room during a search (b), then Training after that search (b).',
    'game_id': '01006A800016E000',
    'scope': 'connection= in hang-trace.log and matches/ for offline Training and the waiting room',
    'start': 'frontend-ready',
    'on_failure': 'stop',
    'setup': SETUP,
    'segments': SEGMENTS,
    'gates': GATES,
    'rows': ROWS,
}

SINGLE_SETUP = [
    {'state': 'one Eden frontend up, game not loaded, the GDB stub set on (defer_load)', 'via': 'hook',
     'ref': 'gdb_rc11.py gdb_on with gdb-port-a.txt and gdb-nohold.txt'},
    {'state': 'the main menu, Smash focused', 'via': 'hook', 'ref': 'qp_steps.py to_main; main gate recover attract-cancel'},
    {'state': 'offline Training through Games & More, P1 on tile 0', 'via': 'segment',
     'ref': 'library main-stage, stage-css, enter-training; qp_steps.py _css_to_tile0'},
    {'state': 'out of Training to the main menu, Smash focused', 'via': 'segment',
     'ref': 'segments quit-focus, quit-confirm (the integration lane); qp_steps.py exit_training, _back_to_main'},
    {'state': 'searching in the Quickplay waiting room', 'via': 'segment',
     'ref': 'segments to-online, to-quickplay, to-css, tile-up, pick (eden/unicode-label); qp_steps.py qp_waiting'},
    {'state': 'Leave Battle, main menu, Training', 'via': 'hook', 'ref': 'qp_steps.py training_after_qp'},
]

if __name__ == '__main__':
    mode = sys.argv[1] if len(sys.argv) > 1 else 'pair'
    out = Path(sys.argv[2]) if len(sys.argv) > 2 else HERE / 'rows.json'
    if mode == 'single':
        PLAN.update(rows=SINGLE, setup=SINGLE_SETUP,
                    about='Deluxe qp-fix 1173bea (NRO 2278c03d, build 0.10.5-1173bead82fa-1794575569) on one LAN '
                          'Quickplay instance, the detector and the tile cursor beside it, the GDB stub on: offline '
                          'Training through Games & More on a fresh boot, out to the main menu, the Quickplay waiting '
                          'room during a search, then Training after that search.')
    out.write_text(json.dumps(PLAN, indent=1) + '\n')
    print(out, mode, sha(out))
