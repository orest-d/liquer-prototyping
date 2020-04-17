#!/usr/bin/python
# -*- coding: utf-8 -*-
'''
Unit tests for LiQuer.
'''
import pytest
from path import *


class TestPath:
    def test_encode_token(self):
        assert encode_token("") == ""
