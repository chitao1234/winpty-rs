# Copyright (c) 2011-2015 Ryan Prichard
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to
# deal in the Software without restriction, including without limitation the
# rights to use, copy, modify, merge, publish, distribute, sublicense, and/or
# sell copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in
# all copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
# FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
# IN THE SOFTWARE.

ALL_TARGETS += build/winpty.dll build/libwinpty.a

$(eval $(call def_mingw_target,libwinpty-dll,-DCOMPILING_WINPTY_DLL))
$(eval $(call def_mingw_target,libwinpty-static,-DWINPTY_STATIC))

LIBWINPTY_OBJECT_STEMS = \
	libwinpty/AgentLocation \
	libwinpty/winpty \
	shared/BackgroundDesktop \
	shared/Buffer \
	shared/DebugClient \
	shared/GenRandom \
	shared/OwnedHandle \
	shared/StringUtil \
	shared/WindowsSecurity \
	shared/WindowsVersion \
	shared/WinptyAssert \
	shared/WinptyException \
	shared/WinptyVersion

LIBWINPTY_DLL_OBJECTS = \
	$(addprefix build/libwinpty-dll/,$(addsuffix .o,$(LIBWINPTY_OBJECT_STEMS)))

LIBWINPTY_STATIC_OBJECTS = \
	$(addprefix build/libwinpty-static/,$(addsuffix .o,$(LIBWINPTY_OBJECT_STEMS)))

build/libwinpty-dll/shared/WinptyVersion.o \
build/libwinpty-static/shared/WinptyVersion.o : build/gen/GenVersion.h

build/winpty.dll : $(LIBWINPTY_DLL_OBJECTS)
	$(info Linking $@)
	@$(MINGW_CXX) $(MINGW_LDFLAGS) -shared -o $@ $^ -Wl,--out-implib,build/winpty.lib

build/libwinpty.a : $(LIBWINPTY_STATIC_OBJECTS)
	$(info Archiving $@)
	@$(MINGW_AR) rcs $@ $^

-include $(LIBWINPTY_DLL_OBJECTS:.o=.d) $(LIBWINPTY_STATIC_OBJECTS:.o=.d)
