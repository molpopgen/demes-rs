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
iterate_epochs(const Deme *deme)
{
    Epoch const *epoch;
    EpochIterator *iterator = demes_deme_epoch_iterator(deme);
    double midpoint, size_at_midpoint, start_time, end_time;
    int code;
    SizeFunction size_function;
    fprintf(stdout, "\tepoch details:\n");
    while ((epoch = demes_epoch_iterator_next(iterator)) != NULL)
        {
            assert(epoch != NULL);
            start_time = demes_epoch_start_time(epoch);
            end_time = demes_epoch_end_time(epoch);
            midpoint = end_time + (start_time - end_time) / 2.0;
            code = demes_epoch_size_at(epoch, midpoint, &size_at_midpoint);
            size_function = demes_epoch_size_function(epoch);
            if (code != 0)
                {
                    abort();
                }
            fprintf(stdout, "\t\tstart time: %lf\n", start_time);
            fprintf(stdout, "\t\tend time: %lf\n", end_time);
            fprintf(stdout, "\t\tstart size: %lf\n", demes_epoch_start_size(epoch));
            fprintf(stdout, "\t\tmidpoint size: %lf\n", size_at_midpoint);
            fprintf(stdout, "\t\tend size: %lf\n", demes_epoch_end_size(epoch));
            switch (size_function)
                {
                case Exponential:
                    fprintf(stdout, "\t\tsize function: exponential\n");
                    break;
                case Linear:
                    fprintf(stdout, "\t\tsize function: linear\n");
                    break;
                case Constant:
                    fprintf(stdout, "\t\tsize function: constant\n");
                    break;
                default:
                    /* demes defines no other size functions but that could change in the future */
                    abort();
                }
        }
    demes_epoch_iterator_deallocate(iterator);
}

void
iterate_deme_ancestors(Graph *const graph, Deme const *const deme)
{
    DemeAncestor const *ancestors;
    DemeAncestorIterator *iterator = demes_deme_ancestor_iterator(deme, graph);
    char *ancestor_name;
    assert(iterator != NULL);

    fprintf(stdout, "\tancestors:\n");
    while ((ancestors = demes_deme_ancestor_iterator_next(iterator)) != NULL)
        {
            ancestor_name = demes_deme_name(ancestors->deme);
            fprintf(stdout, "\t\tname: %s\n", ancestor_name);
            fprintf(stdout, "\t\tproportion: %lf\n", ancestors->proportion);
            demes_c_char_deallocate(ancestor_name);
        }

    demes_deme_ancestor_iterator_deallocate(iterator);
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
            iterate_deme_ancestors(graph, deme);
            iterate_epochs(deme);
        }

    demes_deme_iterator_deallocate(iterator);
}

void
iterate_pulses(Graph const *const graph)
{
    PulseIterator *iterator = demes_graph_pulse_iterator(graph);
    Pulse const *pulse;
    uintptr_t num_source_demes, j;
    char *deme_name;
    double const *pulse_proportions;
    double time;

    while ((pulse = demes_pulse_iterator_next(iterator)) != NULL)
        {
            time = demes_pulse_time(pulse);
            fprintf(stdout, "\tTime of pulse: %lf\n", time);
            num_source_demes = demes_pulse_num_sources(pulse);
            pulse_proportions = demes_pulse_proportions(pulse);
            for (j = 0; j < num_source_demes; ++j)
                {
                    deme_name = demes_pulse_source(pulse, j);
                    assert(deme_name != NULL);
                    fprintf(stdout, "\tsource: %s, proportion: %lf\n", deme_name,
                            pulse_proportions[j]);
                    demes_c_char_deallocate(deme_name);
                    deme_name = demes_pulse_dest(pulse);
                    fprintf(stdout, "\tdestination: %s\n", deme_name);
                    demes_c_char_deallocate(deme_name);
                }
        }
    demes_pulse_iterator_deallocate(iterator);
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
    iterate_pulses(graph);

    demes_error_deallocate(error);
    if (graph != NULL)
        {
            demes_graph_deallocate(graph);
        }
}
