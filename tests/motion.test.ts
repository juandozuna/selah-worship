import { test } from 'node:test';
import assert from 'node:assert/strict';
import { seed } from '../src/data';
import { resolveMotion, snapshot, validateProject, CUT_MOTION } from '../src/model';
import { ease, unitRanks } from '../src/motion';

test('easing matches the Rust table',()=>{
  // Same literals as easing_table_matches_web_implementation in crates/renderer.
  for(const [kind,t,expected] of [['linear',.25,.25],['ease-in',.5,.125],['ease-out',.5,.875],['ease-in-out',.25,.0625],['ease-in-out',.75,.9375]] as const) assert.ok(Math.abs(ease(kind,t)-expected)<1e-9,`${kind} ${t}`);
  assert.equal(ease('spring',1),1);
});
test('stagger order ranks are a permutation and follow the order',()=>{
  assert.deepEqual(unitRanks(4,'forward'),[0,1,2,3]);
  assert.deepEqual(unitRanks(4,'backward'),[3,2,1,0]);
  assert.equal(unitRanks(5,'center')[2],0);
  assert.deepEqual([...unitRanks(6,'random')].sort(),[0,1,2,3,4,5]);
});
test('block motion overrides song motion overrides the service default',()=>{
  const p=structuredClone(seed),song=p.library[0];
  assert.equal(resolveMotion(p,song,song.blocks[0]).transition.kind,'fade');
  song.motion={transition:{...CUT_MOTION.transition,kind:'wipe'}};
  assert.equal(resolveMotion(p,song,song.blocks[0]).transition.kind,'wipe');
  song.blocks[0].motion={text:{...CUT_MOTION.text,kind:'typewriter'}};
  const m=resolveMotion(p,song,song.blocks[0]);
  assert.equal(m.transition.kind,'wipe');assert.equal(m.text.kind,'typewriter');
  assert.equal(snapshot(p,song,0,false).motion,null);
  assert.equal(snapshot(p,song,0).motion?.text.kind,'typewriter');
});
test('services saved before motion existed still validate and behave as a cut',()=>{
  const old:any=structuredClone(seed);delete old.motion;
  validateProject(old);
  assert.equal(resolveMotion(old,old.library[0]).transition.kind,'cut');
  const bad=structuredClone(seed);bad.motion!.transition.durationMs=5;
  assert.throws(()=>validateProject(bad));
});
