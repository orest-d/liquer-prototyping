from typing import NamedTuple
from pyparsing import Literal, Word, alphas, alphanums, delimitedList, Regex, OneOrMore, Group, Combine, ZeroOrMore, Optional, Forward
from urllib.parse import quote, unquote

class Item(NamedTuple):
    name:str
    parameters:list
    def __str__(self):
        return "-".join(str(x) for x in [self.name]+self.parameters)

    def parameters_decoded(self):
        return [x.decode_parameter() for x in self.parameters]

class Path(NamedTuple):
    items:list
    def __str__(self):
        return "/".join(str(x) for x in self.items)
    def as_subpath(self):
        return SubPath(self.items)

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

item_separator = Literal("/").suppress()
separator = Literal("-").suppress()
empty_parameter = Literal("-").setParseAction(lambda s,loc,toks: [""])
identifier = Word(alphas+"_", alphanums+"_").setName("identifier")
parameter_fragment = Regex("[^/~-]+").setWhitespaceChars("")

escape = Literal("~~").setParseAction(lambda s,loc,toks: ["~"])
http_entity = Literal("~h").setParseAction(lambda s,loc,toks: ["http://"])
file_entity = Literal("~f").setParseAction(lambda s,loc,toks: ["file://"])
minus_entity = (Literal("~_")|Literal("~-")).setParseAction(lambda s,loc,toks: ["-"])
space_entity = Literal("~.").setParseAction(lambda s,loc,toks: [" "])
end_entity=(Literal("~e~")|Literal("~end~")).suppress()
path_start_entity = (Literal("~p~")|Literal("~path~")).suppress()
path = Forward()
path_entity = (path_start_entity + path + end_entity).setParseAction(lambda s,loc,toks: [toks[0].as_subpath()])

entities = (escape|http_entity|file_entity|minus_entity|space_entity|path_entity)#.setParseAction(lambda s,loc,tok:[f"E{tok}"])

parameter = OneOrMore(parameter_fragment|entities)

item = (identifier + Group(ZeroOrMore(
    separator + Optional(parameter).setParseAction(lambda s,loc,toks:[Parameter(list(toks))]))
    )).setParseAction(
        lambda s, loc, toks: [Item(toks[0],list(toks[1]))])
#        lambda s,loc,tok:[f"PATH_ENTITY{tok}"])

path <<= delimitedList(item,"/").setParseAction(
        lambda s, loc, toks: [Path(list(toks))])

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
""".split():
        p = path.parseString(text,True)
        print(f"path       %-10s :"%text)
        for item in p[0].items:
            print("  -",repr(item))
        print(f"  str            :{str(p[0])}")
        print(f"  parameters     :")
        for item in p[0].items:
            print("  -",repr(item.parameters_decoded()))
