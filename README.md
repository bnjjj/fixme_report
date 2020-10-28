# FIXME report

FIXME report is a simple tool to automate your issues creation directly from `// TODO` and `// FIXME` comments in your codebase.
Too often you write these comments and forget to delete them or take them into account for your project's next milestone. Note that you can also assign your issues to an assignee by writing a comment with the following syntax `// TODO (@bnjjj): clean this function`
At the current stage FIXME report only supports GitHub issue creation, but take a look at our [roadmap](#roadmap) to know the next platforms supported.

## Configuration

Save your configuration in a JSON file:

```json
{
    "type": "github",
    "repository": "bnjjj/fixme_report",
    "token": "PERSONAL_TOKEN_FROM_GITHUB",
    "username": "bnjjj",
    "url": "https://github.com"
}
```

You can save it into a file named `fixme_settings.json` in the same directory of your project. If you want to have it somewhere else you can use a flag to indicate the path to your configuration file as following `fixme_report -c $HOME/my_settings.json`.

## Usage

The default behavior is to read a patchset (git diff) directly from stdin. If you want to specify a `.patch` file you can do it by adding the following parameter `-f=yourFile.patch` to the CLI.

Example: create issues based on your last commit:

```bash
$ git diff HEAD^1 HEAD | fixme_report
```

The purpose is to add this CLI to your CI/CD pipeline in order to launch it on every commit merged to master.


```bash
USAGE:
    fixme_report [FLAGS] [OPTIONS]

FLAGS:
    -d, --dry-run    display issues to create without creating them
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config <config-file>               giving configuration file, default is fixme_settings.json in the current
                                             directory
    -f, --file <file>                        giving patchset file instead of parsing it via stdin
    -m, --fixme-template <fixme-template>    giving template (handlebars) file for fixme cases (OPTIONAL)
    -t, --todo-template <todo-template>      giving template (handlebars) file for todo cases (OPTIONAL)
```

Here is an example of an issue created with [this of git diff](samples/sample_with_todo):

![example issue](example_issue.png)

+ You can also give for each kind of comment (TODO, FIXME) create a template for to create description of issue as you want. [Here is an example file](./example_todo.tmpl). You just have to use `--todo-template` and/or `--fixme-template` to use your specified file. In these templates you can access 3 different variables as `file`, `line` and `details` to have the comment details. It must be a [handlebars](https://handlebarsjs.com/) compliant template.


## Roadmap

+ Add support of Bitbucket Cloud
+ Add support of Bitbucket Server
+ Add support of Gitlab
+ Add support of Jira
+ Add more tests
+ Better lib architecture
