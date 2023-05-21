import multiprocessing
from multiprocessing.managers import BaseManager, SyncManager
import random
import time


class MyManager(SyncManager):
    pass

    

_workers=[]

def execute_job(jq, worker_id, job, channel):
    try:
        i=int(job[3:])
    except:
        i=0

    if i:
        dependency_job = f"Job{i-1}"
        print(f"{job} depends on {dependency_job}")
        jq.request(worker_id, dependency_job)
    time.sleep(random.random()*0.1)
#    if type(dependency) == str:
#        dependency_value = dependency
#    else:
#        dependency_value = dependency.get()
    if i:
        dependency_value = channel.recv()
        print(f"Dependency {dependency_job} received {dependency_value}")
    else:
        dependency_value = "~"
    
    return "Result-"+job+"("+dependency_value+")"

def worker_process(jq, worker_id, channel):
    while True:
        job = jq.get_job()
        print(f"Worker {worker_id} got job {job}")
        time.sleep(0.2)
        result = execute_job(jq, worker_id, job, channel)
        jq.set(job, result, worker_id)
        print(f"Worker {worker_id} finished job {job}")

def new_worker(jq):
    global _workers
    (parent_channel, child_channel) = multiprocessing.Pipe()
    worker_id = f"p{len(_workers)}"
    p = multiprocessing.Process(target=worker_process, args=(jq, worker_id, child_channel))
    p.start()
    jq.register_channel(worker_id, parent_channel)
    _workers.append(p)

def start_workers(jq):
    for i in range(2):
        new_worker(jq)

class JobQueue:
    def __init__(self):
        self.queue = multiprocessing.Queue()
        self.results = {}
        self.requests = {}
        self.channels = {}

    def register_channel(self, worker_id, channel):
        self.channels[worker_id] = channel

    def submit(self, job):
        self.queue.put(job)

#    def request(self, job):
#        if job in self.results:
#            return (self.results[job])
#        q = multiprocessing.Queue()
#        if job not in self.requests:
#            self.requests[job] = []
#
#        self.requests[job].append(q)
#        return q

    def request(self, worker_id, job):
        if job in self.results:
            print(f"request {job} available immediately: {self.results[job]}")
            self.channels[worker_id].send(self.results[job])
        if job not in self.requests:
            self.requests[job] = []
        self.requests[job].append(worker_id)

    def get_job(self):
        return self.queue.get()

    def set(self, job, result, worker_id):
        print(f"set {job} {result} {worker_id}")
        print(f"  called from {multiprocessing.current_process().name}")
        self.results[job] = result
#        for q in self.requests.get(job, []):
#            q.put(result)
#        del self.requests[job]
        if job in self.requests:
            for w in self.requests.get(job, []):
                print(f"  {job} {result} from {worker_id} send to {w}")
                self.channels[w].send(result)
            del self.requests[job]

    def get(self, job):
        print(f"get {job}")
        print(f"  called from {multiprocessing.current_process().name}")
        #print(self.results)
        return self.results.get(job, None)


#MyManager.register('JobQueue', JobQueue, exposed=('submit', 'get', 'set'))
MyManager.register('JobQueue', JobQueue)

def run(aaa):
    print(f"run {aaa}")
    print(f"  called from {multiprocessing.current_process().name}")
    aaa.set("RUN")

if __name__ == '__main__':
    with MyManager() as manager:
        jq = manager.JobQueue()
        start_workers(jq)
        #jq.set("X", "ResultX", "MAIN")
        for i in range(10):
            jq.submit(f"Job{i}")
            time.sleep(0.1)
        print("---")
        time.sleep(0.5)
        print("---")
        for i in range(10):
            print(f"Result {i}: {jq.get(f'Job{i}')}")
        print()
        input("Press Enter to continue...")
        for worker in _workers:
            worker.terminate()