#define DEMES_FFI = 1

#include <stdio.h>
#include <stdlib.h>
#include <assert.h>
#include <string.h>
#include <demes.h>

void
handle_error(int rv, FFIError *error, Graph *graph)
{
    char *error_msg = NULL;
    if (rv != 0)
        {
            assert(demes_error_has_error(error));
            error_msg = demes_error_message(error);
            assert(error_msg != NULL);
            fprintf(stderr, "%s\n", error_msg);
            demes_c_char_deallocate(error_msg);
            demes_error_deallocate(error);
            if (graph != NULL)
                {
                    demes_graph_deallocate(graph);
                }
            exit(1);
        }
}

void
iterate_epochs(const Deme *)
{
    abort();
}

void
iterate_demes(Graph *graph)
{
    Deme const *deme;
    char *name;
    DemeIterator *iterator = demes_graph_deme_iterator(graph);

    while ((deme = demes_deme_iterator_next(iterator)) != NULL)
        {
            name = demes_deme_name(deme);
            fprintf(stdout, "%s:\n", name);
            fprintf(stdout, "\tstart time: %lf\n", demes_deme_start_time(deme));
            fprintf(stdout, "\tend time: %lf\n", demes_deme_end_time(deme));
            fprintf(stdout, "\tstart size: %lf\n", demes_deme_start_size(deme));
            fprintf(stdout, "\tend size: %lf\n", demes_deme_end_size(deme));
            demes_c_char_deallocate(name);
            iterate_epochs(deme);
        }

    demes_deme_iterator_deallocate(iterator);
}

int
main(int argc, char **argv)
{
    FFIError *error = NULL;
    Graph *graph = NULL;
    int rv;

    if (argc != 2)
        {
            fprintf(stderr, "usage: example filename\n");
            exit(1);
        }

    error = demes_error_allocate();

    rv = demes_graph_load_from_file(argv[1], error, &graph);
    handle_error(rv, error, graph);

    iterate_demes(graph);

    demes_error_deallocate(error);
    if (graph != NULL)
        {
            demes_graph_deallocate(graph);
        }
}
