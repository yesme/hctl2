// Claude Code's native mods API; loaded for this Agency session only.
const root = __HCTL_ROOT__;
let job;
export function register(on) {
  on('session.start', async ($, e, next) => {
    await $.fs.write(root + '/session.json', JSON.stringify({session: await $.session.id()}));
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
