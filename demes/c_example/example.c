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
    size_t num_epochs = demes_deme_num_epochs(deme);
    size_t i;
    int code;
    double midpoint, size_at_midpoint, start_time, end_time;
    for (i = 0; i < num_epochs; ++i)
        {
            epoch = demes_deme_epoch(deme, i);
            assert(epoch != NULL);
            start_time = demes_epoch_start_time(epoch);
            end_time = demes_epoch_end_time(epoch);
            midpoint = end_time + (start_time - end_time) / 2.0;
            code = demes_epoch_size_at(epoch, midpoint, &size_at_midpoint);
            if (code != 0)
                {
                    abort();
                }
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
            ancestor = demes_graph_deme(graph, ancestor_indexes[i]);
            assert(ancestor != NULL);
            ancestor_name = demes_deme_name(ancestor);
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

    if (demes_error_has_error(error))
        {
            abort();
        }

    num_demes = demes_graph_num_demes(graph);
    for (i = 0; i < num_demes; ++i)
        {
            deme = demes_graph_deme(graph, i);
            assert(deme != NULL);
            num_epochs = demes_deme_num_epochs(deme);
            deme_name = demes_deme_name(deme);
            assert(deme_name != NULL);

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

void
iterate_pulses(Graph *graph)
{
    uintptr_t num_pulses, num_source_demes, i, j;
    char *deme_name;
    double const *pulse_proportions;
    double time;
    Pulse const *pulse;

    num_pulses = demes_graph_num_pulses(graph);

    if (num_pulses > 0)
        {
            fprintf(stdout, "Pulses:\n");
        }

    for (i = 0; i < num_pulses; ++i)
        {
            pulse = demes_graph_pulse(graph, i);
            assert(pulse != NULL);
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
}

void
iterate_migrations(Graph *graph)
{
    uintptr_t num_migrations, i;
    AsymmetricMigration const *migration;
    double rate, time;
    char *deme_name, *deme_name2;
    Deme const *deme;

    num_migrations = demes_graph_num_migrations(graph);
    if (num_migrations > 0)
        {
            fprintf(stdout, "Migrations:\n");
        }
    for (i = 0; i < num_migrations; ++i)
        {
            migration = demes_graph_migration(graph, i);
            deme_name = demes_asymmetric_migration_source(migration);
            fprintf(stdout, "\tsource: %s\n", deme_name);
            /* Get the source deme using its name */
            deme = demes_graph_deme_from_name(graph, deme_name);
            /* Get its name */
            deme_name2 = demes_deme_name(deme);
            /* Should be the same...! */
            assert(strcmp(deme_name, deme_name2) == 0);
            demes_c_char_deallocate(deme_name);
            demes_c_char_deallocate(deme_name2);
            deme_name = demes_asymmetric_migration_dest(migration);
            fprintf(stdout, "\tdest: %s\n", deme_name);
            demes_c_char_deallocate(deme_name);
            rate = demes_asymmetric_migration_rate(migration);
            fprintf(stdout, "\trate: %lf\n", rate);
            time = demes_asymmetric_migration_start_time(migration);
            fprintf(stdout, "\tstart time: %lf\n", time);
            time = demes_asymmetric_migration_end_time(migration);
            fprintf(stdout, "\tend time: %lf\n", time);
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
    iterate_pulses(graph);
    iterate_migrations(graph);

    demes_error_deallocate(error);
    if (graph != NULL)
        {
            demes_graph_deallocate(graph);
        }
}
