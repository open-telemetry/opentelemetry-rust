#!/bin/bash

packages=(
    "opentelemetry"
    "opentelemetry-http"
    "opentelemetry-semantic-conventions"
    "opentelemetry-jaeger-propagator"
    "opentelemetry-sdk"    
    "opentelemetry-proto"
    "opentelemetry-otlp"
    "opentelemetry-stdout"
    "opentelemetry-zipkin"
    "opentelemetry-prometheus"
    "opentelemetry-appender-log"
    "opentelemetry-appender-tracing"

    # Add more packages as needed, in the right order. A package should only be published after all it's dependencies have been published
)

# Set the current directory to one level above the scripts directory
current_dir=$(pwd)/..
cd "$current_dir"  # Change to the current directory

# Iterate over the list of packages
for package in "${packages[@]}"; do
    if [ -d "$package" ]; then
        echo "=================================================="
        echo "Processing package: $package"
        cd "$package"  # Change to the directory of package

        # Extract the name and version from Cargo.toml
        name=$(grep -m1 '^name =' Cargo.toml | cut -d'"' -f2)
        version=$(grep -m1 '^version =' Cargo.toml | cut -d'"' -f2)

        if [[ -n "$name" && -n "$version" ]]; then
            echo "Found package $name with version $version" in cargo.toml
            
            # Tag the version in git
            tag="${name}-${version}"
            tag_message="${name} ${version} release."
            # uncomment the following lines after verifying all looks good.
            # git tag -a "$tag" -m "\"$tag_message\""
            # git push origin "$tag"
            echo "git tag -a "$tag" -m "$tag_message""

            # Run cargo publish
            # uncomment the following line after verifying all looks good.
            # cargo publish
            echo "Published $name $version"
        else
            echo "Error: Unable to extract name or version from Cargo.toml in $package"
        fi

        cd "$current_dir"  # Return to the original directory
        echo "Sleeping for 15 seconds before next package..."
        sleep 15  # Sleep for 15 seconds to allow crates.io to index the previous package
    else
        echo "Skipping: $package is not a valid package directory."
    fi
done

echo "Finished publishing all packages."
