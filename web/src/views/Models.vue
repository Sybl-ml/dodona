<template>
  <b-container fluid="md">
    <b-row>
      <b-col>
        <h1>Models</h1>
      </b-col>
    </b-row>

    <hr />

    <model-card
      v-for="(model, index) in model_data"
      :key="index"
      :model="model"
      :i="index"
    />

    <b-row class="justify-content-center">
      <b-col xs="12" lg="6">
        <b-button
          block
          class="mb-4 shadow"
          v-b-toggle.collapse-new
          variant="primary"
          style="border: none"
          onfocus="this.blur();"
        >
          <b-icon-plus-circle-fill></b-icon-plus-circle-fill>
          Add New Model
        </b-button>
      </b-col>
    </b-row>
    <b-row class="justify-content-center">
      <b-col lg="7">
        <b-collapse id="collapse-new" class="mb-4 nodeExpansion">
          <b-card
            class="mb-4 shadow"
            no-body
            style="border: none"
            onfocus="this.blur();"
          >
            <b-card-body title-tag="h5">
              <b-card-title>Connect a New Model</b-card-title>
              <p>
                Download our Python Module from PyPI
              </p>
              <b-row class="justify-content-center">
                <b-col >
                  <b-card class="shadow">
                    <b-row>
                      <b-col xl="11">
                        <code>{{ cli_deps }}</code>
                        <br />
                        <code>{{ cli_code }}</code>
                      </b-col>
                      <b-col style="text-align: right" lg="1">
                        <b-button
                          no-body
                          size="sm"
                          variant="dark"
                          onfocus="this.blur();"
                        >
                          <b-icon-clipboard-plus
                            id="clipboard_btn1"
                            @click="copy(cli_deps, cli_code)"
                          ></b-icon-clipboard-plus>
                          <b-tooltip target="clipboard_btn1" variant="primary" placement="right" triggers="click">Copied!</b-tooltip>
                        </b-button>
                      </b-col>
                    </b-row>
                  </b-card>
                </b-col>
              </b-row>
              <br />
              <p>
                Run the following command to add a new Model
              </p>
              <b-row class="justify-content-center">
                <b-col >
                  <b-card class="shadow">
                    <b-row>
                      <b-col xl="11">
                        <code>{{ cli_setup }}</code>
                      </b-col>
                      <b-col style="text-align: right" lg="1">
                        <b-button
                          no-body
                          size="sm"
                          variant="dark"
                          onfocus="this.blur();"
                        >
                          <b-icon-clipboard-plus
                            id="clipboard_btn2"
                            @click="copy(cli_setup)"
                          ></b-icon-clipboard-plus>
                          <b-tooltip target="clipboard_btn2" variant="primary" placement="right" triggers="click">Copied!</b-tooltip>
                        </b-button>
                      </b-col>
                    </b-row>
                  </b-card>
                </b-col>
              </b-row>
              <br />
              <p>
                For more information, visit the <b><a href="https://www.notion.so/Register-a-Model-f67a613d1cbe4075b2fd72cb3005410e" target="_blank">guide</a></b>
              </p>
            </b-card-body>
          </b-card>
        </b-collapse>
      </b-col>
    </b-row>
    <speedometer />
  </b-container>
</template>

<script>
import ModelCard from "@/components/ModelCard";
import Speedometer from "@/components/charts/Speedometer";


export default {
  name: "Models",
  data() {
    return {
      auth_token: "",
      error: false,
      cli_deps: "pip install pandas pyOpenSSL python-dotenv xdg numpy zenlog",
      cli_code: "pip install -i https://test.pypi.org/simple/ syblmallus",
      cli_setup: "python3 -m sybl authenticate",
    };
  },
  components: {
    ModelCard,
    Speedometer,
  },
  async created() {
    this.$store.dispatch("getModels");
  },
  methods: {
    async copy(...lines) {
      // Concatenate all the arguments
      let text = lines.reduce((previous, current) => {
        return previous + "\n" + current;
      });

      await navigator.clipboard.writeText(text);
    },
  },
  computed: {
    validation() {
      return !this.error;
    },
    model_data() {
      return this.$store.state.models.models;
    },
  },
};
</script>
