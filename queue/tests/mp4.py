# asynchronous task queue
# task is represented as a string query
# Task can be submitted to the queue by a submit method returning a future.
# Submission starts a new python process evaluating the task.
import asyncio
from multiprocessing import Process

def init(local):
    print ("init",getpid())
    manager = MyManager()
    manager.start()    
    print ("init manager=", manager, getpid())

    if local:
        d=dict(local_cache=LocalCacheObject,arg=["LOCAL"])
    else:
        d=dict(central_cache=manager.CentralCache("CENTRAL"))
    print ("init d=",d,getpid())
    return manager.Queue(), d

class TaskQueue:
    def __init__(self,local=True):
        self.q, self.d=init(local)
        self.processes=[]
        self.local=local
    def submit(self,task):
        p=Process(target=work,args=(self.q,self.d,task))
        p.start()
        self.processes.append(p)
        return p
    def wait(self):
        for p in self.processes:
            p.join()
        self.processes=[]
    def close(self):
        if self.local:
            self.wait()
        else:
            self.q.close()
            self.q.join_thread()
        self.q=None
        self.d=None
    def __del__(self):
        self.close()    


# Alternative asynchronous task queue implemented using asyncio.
# Task is represented as a string query
# Task can be submitted to the queue by a submit method returning a future.
# Submission starts a new python process evaluating the task.
class AsyncQueue:
    def __init__(self,local=True):
        self.q, self.d=init(local)
        self.loop=asyncio.get_event_loop()
        self.local=local
    def submit(self,task):
        return self.loop.run_in_executor(None,work,self.q,self.d,task)
    def wait(self):
        pass
    def close(self):
        if self.local:
            self.wait()
        else:
            self.q.close()
            self.q.join_thread()
        self.q=None
        self.d=None
    def __del__(self):
        self.close()



# Example of use asyncio subprocess

import asyncio
import subprocess

async def get_result_from_process(command):
    proc = await asyncio.create_subprocess_shell(
        command,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE)

    stdout, stderr = await proc.communicate()
    return stdout

async def main():
    result = await get_result_from_process('echo "Hello World"')
    print(result.decode('utf-8'))

asyncio.run(main())

# Example of use multiprocessing
import multiprocessing
from concurrent.futures import ProcessPoolExecutor

def get_result_from_process(command):
    proc = multiprocessing.Process(target=command)
    proc.start()
    proc.join()
    return proc.stdout

if __name__ == '__main__':
    with ProcessPoolExecutor() as executor:
        future = executor.submit(get_result_from_process, 'echo "Hello World"')
        result = future.result()
        print(result.decode('utf-8'))