import multiprocessing as mp
from multiprocessing.managers import BaseManager
from os import getpid
from time import sleep
import queue


class LocalCacheObject:
    def __init__(self,name):
        print(f"LocalCacheObject({name}  {id(self)}) constructor at {getpid()}")
        self.name=name
    def call(self,message):
        print(f"Local cache object {self.name}  {id(self)} at {getpid()} got {message}")

class CentralCacheObject:
    def __init__(self,name):
        print(f"CentralCacheObject({name}  {id(self)}) constructor at {getpid()}")
        self.name=name
    def call(self,message):
        print(f"Central cache object {self.name}  {id(self)} at {getpid()} got {message}")


def get_cache(d):
    if "central_cache" in d:
        return d["central_cache"]
    if "local_cache" in d:
        f=d["local_cache"]
        arg=d.get("arg",[])
        if arg is None:
            arg=[]
        return f(*arg)
    return LocalCacheObject("default")

def register_cache_constructor(ns,f):
    print("register_cache_constructor 1",ns,getpid())
    ns.cache_constructor=f
    print("register_cache_constructor 2",ns,getpid())
    return "register_cache_constructor DONE"

class MyManager(BaseManager):
    pass

MyManager.register("CentralCache",CentralCacheObject)
MyManager.register("Queue",queue.Queue)

def init(local=True):
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


def work(q,d,message):
    print ("work ", message, d, getpid())
    cache=get_cache(d)
    cache.call(f"Hello {message} from {getpid()}")
    for i in range(3):
        print(f"{message} Waiting for {i} ({getpid()})")
        task=q.get()
        cache.call(f"Task {task} ({message} from {getpid()})")
        sleep(0.1)
    return f"work DONE {getpid()}"

def submit(q):
    for i in range(10):
        print(f"queue {i}")
        q.put(i)
        sleep(0.05)

if __name__=="__main__":
    print("MAIN",getpid())
    q,d = init(False)
    if True:
        pool = mp.Pool(4)
        pool.apply_async(work,[q,d,"A"], callback=print)
        pool.apply_async(work,[q,d,"B"], callback=print)
        pool.close()
        submit(q)
        pool.join()
    else:
        a=mp.Process(target=work, args=[q,d,"AA"])
        b=mp.Process(target=work, args=[q,d,"BB"])
        a.start()
        b.start()
        submit(q)
        a.join()
        b.join()

#    q.join()
#    while True:
#        sleep(1)

 

