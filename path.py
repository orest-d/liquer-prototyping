from typing import NamedTuple
from pyparsing import Literal, Word, alphas, alphanums, nums, delimitedList, Regex, OneOrMore, Group, Combine, ZeroOrMore, Optional, Forward
from urllib.parse import quote, unquote

class Item(NamedTuple):
    name:str
    parameters:list
    def __str__(self):
        return "-".join(str(x) for x in [self.name]+self.parameters)

    def parameters_decoded(self):
        return [x.decode_parameter() for x in self.parameters]

class Resource(NamedTuple):
    name:str
    def __str__(self):
        return self.name

class Path(NamedTuple):
    items:list
    start_slash:bool
    end_slash:bool

    def __str__(self):
        return self.encode()
    def encode_inner(self):
        return "/".join(str(x) for x in self.items)
    def encode(self):
        res = "/" if self.is_absolute else ""
        res += self.encode_inner()
        res += "/" if self.is_dir else ""
        return res        
    def as_subpath(self):
        return SubPath(self.items, self.start_slash, self.end_slash)

class SubPath(Path):
    def __str__(self):
        return "~path~"+("/".join(str(x) for x in self.items))+"~end~"
    def decode_parameter(self):
        return "/".join(str(x) for x in self.items)

class Parameter(NamedTuple):
    items:list
    def __str__(self):
        return "".join(str(x) for x in self.items)
    def decode_parameter(self):
        txt=""
        for x in self.items:
            if type(x) is str:
                txt+=x
            elif isinstance(x, SubPath):
                txt+=x.decode_parameter()
            else:
                raise Exception(f"Unknonwn parameter part: {repr(x)}")
        return txt

class Section(NamedTuple):
    name:str
    parameters:list
    path:Path

class SectionSequence(NamedTuple):
    items:Section

item_separator = Literal("/").suppress()
separator = Literal("-").suppress()
empty_parameter = Literal("-").setParseAction(lambda s,loc,toks: [""])
identifier = Word(alphas+"_", alphanums+"_").setName("identifier")
resource1 = Regex(r"[a-zA-Z0-9_]\.[a-zA-Z0-9._-]*").setParseAction(lambda s,loc,toks: Resource(toks[0]))
resource2 = Regex(r"\.[a-zA-Z0-9._-]+").setParseAction(lambda s,loc,toks: Resource(toks[0][1:]))
resources = 
parameter_fragment = Regex("[^/~-]+").setWhitespaceChars("")

escape = Literal("~~").setParseAction(lambda s,loc,toks: ["~"])
minus_num_entity = Regex("~[0-9]+").setParseAction(lambda s,loc,toks: ["-"+toks[0][1:]])
http_entity = Literal("~h").setParseAction(lambda s,loc,toks: ["http://"])
file_entity = Literal("~f").setParseAction(lambda s,loc,toks: ["file://"])
minus_entity = (Literal("~_")|Literal("~-")).setParseAction(lambda s,loc,toks: ["-"])
space_entity = Literal("~.").setParseAction(lambda s,loc,toks: [" "])
end_entity=(Literal("~e~")|Literal("~end~")).suppress()
path_start_entity = (Literal("~p~")|Literal("~path~")).suppress()
path = Forward()
path_entity = (path_start_entity + path + end_entity).setParseAction(lambda s,loc,toks: [toks[0].as_subpath()])

entities = (escape|minus_num_entity|http_entity|file_entity|minus_entity|space_entity|path_entity)#.setParseAction(lambda s,loc,tok:[f"E{tok}"])


parameter = OneOrMore(parameter_fragment|entities)

item = (identifier + Group(ZeroOrMore(
    separator + Optional(parameter).setParseAction(lambda s,loc,toks:[Parameter(list(toks))]))
    )).setParseAction(
        lambda s, loc, toks: [Item(toks[0],list(toks[1]))])
#        lambda s,loc,tok:[f"PATH_ENTITY{tok}"])

inner_path = (delimitedList(item,"/") + Optional(Literal("/"))).setParseAction(
        lambda s, loc, toks: [Path(list(toks[:-1]),False,True) if toks[-1]=="/" else Path(list(toks),False,False)])
absolute_path = (Literal("/") + inner_path).setParseAction(
        lambda s, loc, toks: [Path(toks[1].items,True,toks[1].is_dir)])
path <<= inner_path | absolute_path | (Literal("/").setParseAction(lambda s, loc, toks: [Path([],True,False)]))

section = (separator + identifier + Group(ZeroOrMore(
    separator + Optional(parameter).setParseAction(lambda s,loc,toks:[Parameter(list(toks))]))
    + item_separator 
    )+ inner_path).setParseAction(
        lambda s, loc, toks: [Section(toks[0],list(toks[1]),toks[2])])

if __name__ == "__main__":
    print(Item("abc",[1,2,3]), repr(Item("abc",[1,2,3])))
    print(identifier.parseString("abc"))
    print("item       ",item.parseString("abc-~h def~~~_~-~.xxx-~fghi"))
    print("path       ",path.parseString("abc-~h def~~~_~-~.xxx-~fghi"))
    for text in """
    abc/def
    abc
    abc-
    abc--
    abc-a--1-
    abc/def-prefix~p~_subpath/xxx~e~postfix-y
    abc/def-~p~path/p1-~p~subpath/p2~e~~e~
    abc/xxx
    abc/xxx/
    /abc/xxx
    /abc/xxx/
    /
    a-~123
""".split():
        p = path.parseString(text,True)
        print(f"path       %-10s :"%text)
        print("  ",repr(p[0]))
        for item in p[0].items:
            print("  -",repr(item))
        print(f"  str            :{str(p[0])}")
        print(f"  parameters     :")
        for item in p[0].items:
            print("  -",repr(item.parameters_decoded()))

    for text in """
    -sec-1/abc/def
""".split():
        sec = section.parseString(text,True)[0]
        print("sec", sec)
        path = sec.path
        print(f"path       %-10s :"%text)
        print("  ",repr(path))
        for item in path.items:
            print("  -",repr(item))
        print(f"  str            :{str(path)}")
        print(f"  parameters     :")
        for item in path.items:
            print("  -",repr(item.parameters_decoded()))
