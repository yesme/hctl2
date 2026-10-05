// Claude Code's native mods API; loaded for this Agency session only.
const root = __HCTL_ROOT__;
let job;
let draft;
export function register(on) {
  on('session.start', async ($, e, next) => {
    await $.fs.write(root + '/session.json', JSON.stringify({session: await $.session.id()}));
    return next(e);
  });
  on('prompt.edit', async ($, e, next) => {
    const current = JSON.parse(await $.fs.read(root + '/job.json'));
    const key = current.id + ':' + current.digest;
    if (e.inputText && draft !== key) {
      draft = key;
      // Early Esc restores the cancelled prompt. Replace its draft within
      // the native edit; do not discard later chunks of this same paste.
      return next({...e, text: '', cursor: 0, start: 0, end: 0});
    }
    return next(e);
  });
  on('prompt.submit', async ($, e, next) => {
    const current = JSON.parse(await $.fs.read(root + '/job.json'));
    if (e.text.trim() !== current.text) {
      await $.fs.write(root + '/rejected.json', JSON.stringify({
        job: current.id, digest: current.digest, session: await $.session.id(),
      }));
      return {drop: 'Agency prompt does not match this dispatch'};
    }
    return next(e);
  });
  on('turn.start', async ($, e, next) => {
    if (e.agentId !== undefined) return next(e);
    job = JSON.parse(await $.fs.read(root + '/job.json'));
    await $.fs.write(root + '/started.json', JSON.stringify({
      ...e, job: job.id, digest: job.digest, session: await $.session.id(),
    }));
    return next(e);
  });
  on('turn.complete', async ($, e, next) => {
    // A subagent's completion is not this dispatch's returned turn.
    if (e.agentId === undefined && job) {
      await $.fs.write(root + '/returned.json', JSON.stringify({
        ...e, job: job.id, digest: job.digest, session: await $.session.id(),
      }));
    }
    return next(e);
  });
}
