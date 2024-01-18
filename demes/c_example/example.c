#define DEMES_FFI = 1

#include <stdio.h>
#include <stdlib.h>
#include <assert.h>
#include <demes.h>

void
handle_error(int rv, FFIError *error, Graph *graph)
{
    char *error_msg = NULL;
    if (rv != 0)
        {
            assert(demes_error_has_error(error));
            assert((error_msg = demes_error_message(error)) != NULL);
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
iterate_epochs(const Deme *deme)
{
    Epoch const *epoch;
    size_t num_epochs = demes_deme_num_epochs(deme);
    size_t i;
    double midpoint, size_at_midpoint, start_time, end_time;
    for (i = 0; i < num_epochs; ++i)
        {
            assert((epoch = demes_deme_epoch(deme, i)) != NULL);
            start_time = demes_epoch_start_time(epoch);
            end_time = demes_epoch_end_time(epoch);
            midpoint = end_time + (start_time - end_time)/2.0;
            assert(demes_epoch_size_at(epoch, midpoint, &size_at_midpoint) == 0);
            fprintf(stdout, "\t\tstart time: %lf\n", start_time);
            fprintf(stdout, "\t\tend time: %lf\n", end_time);
            fprintf(stdout, "\t\tstart size: %lf\n", demes_epoch_start_size(epoch));
            fprintf(stdout, "\t\tmidpoint size: %lf\n", size_at_midpoint);
            fprintf(stdout, "\t\tend size: %lf\n", demes_epoch_end_size(epoch));
        }
}

void
iterate_ancestors_proportions(const Graph *graph, const Deme *deme)
{
    size_t const *ancestor_indexes;
    Deme const *ancestor;
    double const *ancestor_proportions;
    double proportion;
    char *ancestor_name;
    size_t i, num_ancestors;

    num_ancestors = demes_deme_num_ancestors(deme);
    ancestor_indexes = demes_deme_ancestor_indexes(deme);
    ancestor_proportions = demes_deme_proportions(deme);

    for (i = 0; i < num_ancestors; ++i)
        {
            assert((ancestor = demes_graph_deme(graph, ancestor_indexes[i])) != NULL);
            ancestor_name = demes_deme_name(deme);
            proportion = ancestor_proportions[i];
            fprintf(stdout, "\t \t%s %lf\n", ancestor_name, proportion);
            demes_c_char_deallocate(ancestor_name);
        }
}

void
iterate_demes(Graph *graph, FFIError *error)
{
    size_t i, num_epochs;
    Deme const *deme;
    size_t num_demes;
    char *deme_name = NULL;

    assert(!demes_error_has_error(error));

    num_demes = demes_graph_num_demes(graph);
    for (i = 0; i < num_demes; ++i)
        {
            assert((deme = demes_graph_deme(graph, i)) != NULL);
            num_epochs = demes_deme_num_epochs(deme);
            assert((deme_name = demes_deme_name(deme)) != NULL);
            fprintf(stdout, "deme %ld:\n", i);
            fprintf(stdout, "\tname: %s\n", deme_name);
            fprintf(stdout, "\tno. epochs: %ld\n", num_epochs);
            fprintf(stdout, "\tstart time: %lf\n", demes_deme_start_time(deme));
            fprintf(stdout, "\tend time: %lf\n", demes_deme_end_time(deme));
            fprintf(stdout, "\tstart size: %lf\n", demes_deme_start_size(deme));
            fprintf(stdout, "\tend size: %lf\n", demes_deme_end_size(deme));
            demes_c_char_deallocate(deme_name);
            fprintf(stdout, "\tancestor details:\n");
            iterate_ancestors_proportions(graph, deme);
            fprintf(stdout, "\tepoch details:\n");
            iterate_epochs(deme);
        }
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

    iterate_demes(graph, error);

    demes_error_deallocate(error);
    if (graph != NULL)
        {
            demes_graph_deallocate(graph);
        }
}
