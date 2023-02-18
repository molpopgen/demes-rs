#include <stdint.h>
#include <math.h>
#include <assert.h>
#include <stdio.h>
#include <demes_forward.h>

void
validate_ancestry_proportions(const double* ancestry_proportions,
                              const double* parental_deme_sizes, size_t num_demes)
{
    double sum_ancestry_proportions = 0.0;
    size_t aprop;
    for (aprop = 0; aprop < num_demes; ++aprop)
        {
            assert(ancestry_proportions[aprop] >= 0.0);
            assert(ancestry_proportions[aprop] <= 1.0);
            assert(isfinite(ancestry_proportions[aprop]));
            sum_ancestry_proportions += ancestry_proportions[aprop];
            if (ancestry_proportions[aprop] > 0.0)
                {
                    assert(parental_deme_sizes[aprop] > 0.0);
                }
        }
    assert(sum_ancestry_proportions - 1.0 <= 1e-9);
}

int32_t
process_model(const char* file)
{
    OpaqueForwardGraph* graph = forward_graph_allocate();
    int32_t status;
    double end_time;
    const double* model_time;
    const double* parental_deme_sizes;
    const double* offspring_deme_sizes;
    const double* ancestry_proportions;
    intptr_t num_demes;
    size_t child;
    int32_t rv = 0;

    status = forward_graph_initialize_from_yaml_file(file, 100.0, graph);
    if (status != 0)
        {
            goto out;
        }
    assert(!forward_graph_is_error_state(graph));

    end_time = forward_graph_model_end_time(&status, graph);

    num_demes = forward_graph_number_of_demes(graph);

    if (status != 0)
        {
            goto out;
        }

    status = forward_graph_initialize_time_iteration(graph);
    if (status != 0)
        {
            goto out;
        }
    assert(status == 0);

    for (model_time = forward_graph_iterate_time(graph, &status);
         status == 0 && model_time != NULL;
         model_time = forward_graph_iterate_time(graph, &status))
        {
            /* Update the internal state of the model to model_time */
            status = forward_graph_update_state(*model_time, graph);
            if (status != 0)
                {
                    goto out;
                }
            assert(!forward_graph_is_error_state(graph));
            parental_deme_sizes = forward_graph_parental_deme_sizes(graph, &status);
            if (status != 0)
                {
                    goto out;
                }
            assert(parental_deme_sizes != NULL);
            offspring_deme_sizes = forward_graph_offspring_deme_sizes(graph, &status);
            if (status != 0)
                {
                    goto out;
                }
            if (*model_time < end_time - 1.0)
                {
                    assert(offspring_deme_sizes != NULL);
                    for (child = 0; child < num_demes; ++child)
                        {
                            if (offspring_deme_sizes[child] > 0.0)
                                {
                                    ancestry_proportions
                                        = forward_graph_ancestry_proportions(
                                            child, &status, graph);
                                    if (status != 0)
                                        {
                                            goto out;
                                        }
                                    validate_ancestry_proportions(ancestry_proportions,
                                                                  parental_deme_sizes,
                                                                  num_demes);
                                }
                        }
                }
            else
                {
                    assert(offspring_deme_sizes == NULL);
                }
        }
out:
    if (status < 0)
        {
            rv = status;
            assert(forward_graph_is_error_state(graph));
            fprintf(stdout, "%s\n", forward_graph_get_error_message(graph, &status));
        }
    forward_graph_deallocate(graph);
    return rv;
}

int
main(int argc, char** argv)
{
    int arg = 1;
    int32_t status;
    const char* fn;
    for (; arg < argc; ++arg)
        {
            fn = argv[arg];
            status = process_model(fn);
            fprintf(stdout, "processed %s, final status = %d\n", fn, status);
        }
}
