# Job nomenclature
succeed   - finished successfully producing a result
failed    - finished with an error
crashed   - crashed in an environment rather than in the job execution - i.e. a bug or runtime error; treated as failed
cancelled - finished by request - treated as failed
finished  - don't use
done      - don't use
completed - succees, failed, canceled or crashed
queued    - waiting to be executed
running   - being executed
waiting   - waiting for a rependency
pending   - queued, running or waiting - expected to be completed


**do we need the ability to cancel a job?**

# submit query

- is in cache (or store?) ? => return from cache (promise from cache)

**Cache must always be available. Contains must be fast.**
- is available or failed? => return result (satisfied promise)

**Define and harmonize the naming convention:**
- None, unknown, new, submitted, running, waiting (submitted, running or waiting = pending?), completed or failed (finished?), crashed?; how does it relate to state?

- is submitted (queued, assigned, accepted, running) ? => create promise
- not known (new?) => queue and create promise

# running modes

|mode      | transitive | command | query   | StateType                   | main thread | worker subquery |
|----------|------------|---------|---------|-----------                  |-------------|--------|
|volatile  | yes        | yes     | infered | analogue is is_serializable | allowed     | always |
|mainthread| no         | yes     | infered | could make sense - proxy?   | enforced    | always delegated to server|
|inline    | no         | yes     | infered | like volatile               | allowed     | always |
|normal    | no         | default | infered | default                     | allowed     | allowed |

|designated| maybe       | possibly| special syntax| special proxy         | forbidden   | designated worker |

volatile vs inline:
  volatile needs a new execution every time from the point of first volatile command
  inline does not trigger execution if the result is locally available
  Only volatile always requires new execution
volatile can be combined with mainthread or designated (volatile inline=volatile, volatile normal=volatile)
mainthread is always blocking
inline is weaker volatile => optional
mainthread is required to support sqlite
designated would be required to run (possibly remote) specialized workers, which is largely hypothetical

Hence we would need only normal, volatile, mainthread and mainthread volatile
However - inline should be used to communicate that non-serializable type is returned

instead of inline, it should be indicated that the result will not be serializable.

**Result state type should be indicated in command metadata (optional)**
**is_serializable should be indicated in command in a conservative way (required?)**

# job / query state

- unknown
- new - not in queue
- main_thread - being assigned and executed in the main thread
**This should probably be implemented as a local worker in the main thread**
**do we need two or more states - main_thread_ waiting, pending, running ?**
**If main_thread will be executed immediately, the state will (probably) never be observed in singlethread server** 
**local execution/not serializable/volatile (? volatile may be technically shared if serializable)**
**Should volatile trigger local execution?**
**is_serializable should be part of the StateType** is_serializable should mean both read and write. E.g. when Matplotlib Fig is write only, it is not serializable.
**volatile is transitive, local execution and serializable (in general) is not**
**local_execution is defined on a command or query level. requires_local_execution(query)**
**Local execution is not a good name. inline execution? blocking?**
**Note that inline execution is blocking**
**There might be a difference between main thread and inline execution. quick volatile => inline, sqlite => main thread; mainthread can't be executed in the worker, inline can**
- queued
- submitted - submitted to worker, but not accepted yet; should be accepted shortly after submission
- _rejected by worker_ (can't be)
**Argument: worker ready state in server must be conservatively maintained**
**If job state is not accepted by worker shortly after submitted to worker, action should be taken**
**housekeeping**
  * check if workers are running by regular pings
  * kill and restart faulty workers*
  * check if jobs submitted to worker are accepted by worker
  * pick a job from queue and try to assign to a worker
  * poll workers and handle messages
  * on finished job notify workers
- accepted - accepted by the worker
- running   - being executed
- succeed   - finished successfully producing a result
- failed    - finished with an error
- crashed   - crashed in an environment rather than in the job execution - i.e. a bug or runtime error; treated as failed
- cancelled - finished by request - treated as failed
- completed - succees, failed, canceled or crashed
- waiting   - waiting for a rependency
- pending   - queued, running or waiting - expected to be completed
**How do we identify a crash of a worker?**
**Somehow we command should indicate that it will be running for a long time**
**Crash of a query execution is different than worker crash?**
**Worker crash should trigger a worker restart (housekeeping) and maybe a job restart?**
**We may need a cancel_job methods and messages**

# query submission (top level)

## Job is responsibility of Server
  Job: unknown
- Server: is_cached
  Job: completed or failed
**Cache should support quick return of serialized form**
**MemoryCache supports non-serializable state types, hence there is cachable non-serializable (could be inline)**
**Maybe cachable-nonserializable and memory-cache should be treated as a special case (local cache)**

- Server: Local execution/Not serializable: execute locally

- Server: is_finished (completed or failed) ? => return
  Job: completed or failed
**Do we need to support futures/promisses?**
**Do we need a future? Do we need to support multiple forms?**
**We are going to have a server future and worker future. Possibly a server thread future.**

- Server: is_pending (queued, assigned, accepted, running)
  Job: queued, assigned, accepted, running

- Server: create and queue the query
  Job: queued

- Server: process_queue
  get queue from query and find ready worked
  Server: send SubmitJob
  Job: submitted to worked

  Worker: receive SubmitJob
  Worker: puts the job into a local queue
  Worker: reply WorkerAcceptedJob
  Server: poll - on_job_accepted
  Job: assigned to worker

## Job is responsibility of Worker
- Worker: gets the job from a local queue
- Worker: sends WorkerJobRunning
- Server: on_job_running
- Job: running

- Worker: send progress message to server WorkerProgress
**Do we need multiple progress messages or is one enough?**

- Worker: submit_subquery
- Worker: sends WorkerJobSubquery
- Worker: original job is put into queue?
- Server: on_subquery

Subquery can be:
- ready - in cache, in worker cache                                 => resumes, server is not contacted 
- job is volatile or inline (not serializable)                      => executes in worker
- ready on server and is serializable and cacheable                 => JobReadyInCache(JobCompleted) or JobFailed(JobCompleted)
- ready on server and is serializable and small but not cacheable   => JobResult(JobCompleted)
- ready on server but can't neither cache nor send                  => InlineJob
- pending                                                           => JobPending, later JobCOmpleted instance or InlineJob (?)
- assigned to the requesting worker                                 => AssignJob


- after job worker submits 
