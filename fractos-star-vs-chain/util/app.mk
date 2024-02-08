ifeq ($(TARGET),)
TARGET := $(basename $(realpath .))
endif

SRCDIR := src
OBJDIR := obj
INCDIR := src

# target directory for any libraries built by this makefile
LIBDIR := lib

# should be passed from external caller
TGTDIR := bin

MKPATH := $(dir $(realpath $(firstword $(MAKEFILE_LIST))))

LIBS   := -lm -lpthread -lboost_system -lboost_thread -Wl,--no-as-needed -ldl -lglog
CPPFLAGS ?= $(shell pkg-config caladan-metaos --cflags) $(shell pkg-config caladan-metakernel --cflags)
LDFLAGS ?= $(shell pkg-config caladan-metaos --libs) $(shell pkg-config caladan-metakernel --libs) -lglog -pthread
CXXFLAGS := -std=c++2a -g -fPIC -O3 -Wall # -fsanitize=address

override CPPFLAGS += -D BOOST_THREAD_PROVIDES_FUTURE_WHEN_ALL_WHEN_ANY
override CPPFLAGS += -D BOOST_THREAD_PROVIDES_EXECUTORS
override CPPFLAGS += -D BOOST_THREAD_USES_MOVE

NVCC=/usr/local/cuda/bin/nvcc
NSOURCES := $(shell find $(SRCDIR) -name *.cu -type f)

SOURCES := $(shell find $(SRCDIR) -name *.cpp -type f)
HEADERS := $(shell find $(INCDIR) -name *.h -type f) $(shell find $(INCDIR) -name *.cuh -type f) $(shell find $(INCDIR) -name *.hpp -type f)
OBJECTS := $(subst $(SRCDIR),$(OBJDIR),$(SOURCES:.cpp=.o))
OBJECTS += $(subst $(SRCDIR),$(OBJDIR),$(NSOURCES:.cu=.o))
TGTOBJS := $(addprefix $(OBJDIR)/, $(addsuffix .o, $(TARGET)))

.PHONY: all clean
.PRECIOUS: $(OBJECTS)

all:: $(patsubst %, $(TGTDIR)/%, $(filter-out lib%, $(TARGET)))
all:: $(patsubst %, $(LIBDIR)/%.a, $(filter lib%, $(TARGET)))

$(TGTDIR)/%: $(OBJDIR)/%.o $(filter-out $(TGTOBJS), $(OBJECTS))
	@mkdir -p $(@D)
	@echo "Linking $@..."
	@$(CXX) -pedantic -o $@ $^ $(LDFLAGS) $(LIBS)

$(LIBDIR)/%.a: $(OBJDIR)/%.o $(filter-out $(TGTOBJS), $(OBJECTS))
	@mkdir -p $(@D)
	@echo "Linking $@..."
	@# $(CXX) -shared $(LDFLAGS) -o $@.so $^ $(LIBS)
	@ar rs $@ $^

$(OBJDIR)/%.o: $(SRCDIR)/%.cu $(HEADERS)
	@mkdir -p $(OBJDIR)
	@echo "Building object file $@..."
	@$(NVCC) -g -c -o $@ $< -I$(realpath ../../include)


$(OBJDIR)/%.o: $(SRCDIR)/%.cpp $(HEADERS)
	@mkdir -p $(OBJDIR)
	@echo "Building object file $@..."
	@$(CXX) -g -c $(CPPFLAGS) $(CXXFLAGS) -o $@ $<

clean:
	rm -f $(OBJECTS)
	rm -f $(TGTDIR)/*
	rm -f $(LIBDIR)/*
