#pragma once

#include "transys.h"
#include "giputils.h"

extern "C" {
void *gipsat_new(const void *);

void gipsat_drop(void *);

size_t gipsat_level(void *);

void gipsat_extend(void *);

void gipsat_add_lemma(void *, uint, uint *, uint);

int gipsat_inductive(void *, uint, uint *, uint, int);

class RustVec gipsat_inductive_core(void *);

class RustVec gipsat_get_predecessor(void *);

int gipsat_propagate(void *);

bool gipsat_has_bad(void *);

void gipsat_set_domain(void *, int, uint *, uint);

void gipsat_unset_domain(void *, int);
}

class GipSAT {
    public:
	GipSAT(Transys &transys)
	{
		ptr = gipsat_new(transys.ptr);
	}

	~GipSAT()
	{
		gipsat_drop(ptr);
	}

	size_t level()
	{
		return gipsat_level(ptr);
	}

	void extend()
	{
		gipsat_extend(ptr);
	}

	void add_lemma(uint frame, std::vector<uint> &cube)
	{
		gipsat_add_lemma(ptr, frame, cube.data(), cube.size());
	}

	bool inductive(uint frame, std::vector<uint> &cube, bool strengthen)
	{
		return gipsat_inductive(ptr, frame, cube.data(), cube.size(), strengthen) == 1;
	}

	std::vector<uint> inductive_core()
	{
		RustVec rv = gipsat_inductive_core(ptr);
		std::vector<uint> res;
		uint *data = (uint *)rv.data();
		for (int i = 0; i < rv.size(); ++i) {
			res.push_back(*(data + i));
		}
		return res;
	}

	std::vector<uint> get_predecessor()
	{
		RustVec rv = gipsat_get_predecessor(ptr);
		std::vector<uint> res;
		uint *data = (uint *)rv.data();
		for (int i = 0; i < rv.size(); ++i) {
			res.push_back(*(data + i));
		}
		return res;
	}

	bool propagate()
	{
		return gipsat_propagate(ptr) == 1;
	}

	bool has_bad()
	{
		return gipsat_has_bad(ptr) == 1;
	}

	void set_domain(uint frame, std::vector<uint> &d)
	{
		gipsat_set_domain(ptr, frame, d.data(), d.size());
	}

	void unset_domain(uint frame)
	{
		gipsat_unset_domain(ptr, frame);
	}

    private:
	void *ptr;
};