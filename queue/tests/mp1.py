import multiprocessing as mp
from multiprocessing.managers import BaseManager, SyncManager, Namespace
from os import getpid
from time import sleep


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

class MyPool:
    def set_local_cache(self, cache_constructor, cache_arguments=None):
        self.local_cache=(cache_constructor, cache_arguments)
    def set_central_cache(self, cache):
        self.central_cache=cache

    def get_cache(self):
        c = getattr(self, "central_cache", None)
        if c is not None:
            return c
        c = getattr(self, "local_cache", None)
        if c is not None:
            f,arg=c
            if arg is None:
                arg=[]
            return f(*arg)
        return LocalCacheObject("default")
    def __str__(self):
        c = getattr(self, "central_cache", None)
        if c is not None:
            return f"MyPool with central cache at {getpid()}"
        c = getattr(self, "local_cache", None)
        if c is not None:
            return f"MyPool with local cache at {getpid()}"
        return f"MyPool with default cache at {getpid()}"


class Foo:
    def f(self):
        print('you called Foo.f()')
    def g(self):
        print('you called Foo.g()')
    def _h(self):
        print('you called Foo._h()')

def register_cache_constructor(ns,f):
    print("register_cache_constructor 1",ns,getpid())
    ns.cache_constructor=f
    print("register_cache_constructor 2",ns,getpid())
    return "register_cache_constructor DONE"

class MyManager(BaseManager):
    pass

MyManager.register("Foo1",Foo)
MyManager.register("CentralCache",CentralCacheObject)
MyManager.register("MyPool",MyPool)

def init(local=True):
    print ("init",getpid())
#    manager = mp.Manager()
    manager = MyManager()
    manager.start()    
    print ("init manager=", manager, getpid())
    pool = manager.MyPool()

    print ("init pool1=",pool,getpid())
    if local:
        pool.set_local_cache(LocalCacheObject,["LOCAL"])
    else:
        pool.set_central_cache(manager.CentralCache("CENTRAL"))
#    f=manager.Foo1()
#    f.f()
    print ("init pool2=",pool,getpid())
    print ("init cache:",pool.get_cache())
    return pool


def work(pool,message):
    print ("work ", pool, getpid())
    pool.get_cache().call(message)
    return "work DONE"

if __name__=="__main__":
    print("MAIN",getpid())
    p = init(False)
    if True:
        pool = mp.Pool(4)
        pool.apply_async(work,[p,"A"], callback=print)
        pool.apply_async(work,[p,"B"], callback=print)
        pool.close()
        pool.join()
    else:
        a=mp.Process(target=work, args=[p,"AA"])
        b=mp.Process(target=work, args=[p,"BB"])
        a.start()
        b.start()
        a.join()
        b.join()

#    while True:
#        sleep(1)



