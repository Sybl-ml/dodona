<template>
  <b-container fluid="md">
    <b-row>
      <b-col>
        <h1>Models</h1>
      </b-col>
    </b-row>

    <hr />

    <model-card
      v-for="(m, index) in model_data"
      :key="index"
      :data="m"
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
          Add New Node
        </b-button>
      </b-col>
    </b-row>
    <b-row class="justify-content-center">
      <b-col xs="12" lg="7">
        <b-collapse id="collapse-new" class="mb-4 nodeExpansion">
          <b-card
            class="mb-4 shadow"
            no-body
            style="border: none"
            onfocus="this.blur();"
          >
            <b-card-body title-tag="h5">
              <b-card-title>Connect a New Node</b-card-title>
              <p>
                Execute the below script on your host compute node to get the
                Sybl-CLI
              </p>
              <b-row class="justify-content-center">
                <b-col xs="12" lg="10">
                  <b-card class="shadow">
                    <b-row>
                      <b-col md="10">
                        <code>{{ cli_code }}</code>
                      </b-col>
                      <b-col style="text-align: right" md="2">
                        <b-button
                          no-body
                          variant="dark"
                          style="
                            background: none;
                            border: none;
                            margin: 0;
                            padding: 0;
                          "
                          onfocus="this.blur();"
                        >
                          <b-icon-clipboard-plus
                            @click="copy(cli_code)"
                          ></b-icon-clipboard-plus>
                        </b-button>
                      </b-col>
                    </b-row>
                  </b-card>
                </b-col>
              </b-row>
              <br />
              <p>
                Then run the following commands to connect your compute node the
                Sybl servers
              </p>
              <b-row class="justify-content-center">
                <b-col xs="12" lg="10">
                  <b-card class="shadow">
                    <b-row>
                      <b-col md="10">
                        <code>{{ cli_setup }}</code>
                      </b-col>
                      <b-col style="text-align: right" md="2">
                        <b-button
                          no-body
                          variant="dark"
                          style="
                            background: none;
                            border: none;
                            margin: 0;
                            padding: 0;
                          "
                          onfocus="this.blur();"
                        >
                          <b-icon-clipboard-plus
                            @click="copy(cli_setup)"
                          ></b-icon-clipboard-plus>
                        </b-button>
                      </b-col>
                    </b-row>
                  </b-card>
                </b-col>
              </b-row>
              <br />
            </b-card-body>
          </b-card>
        </b-collapse>
      </b-col>
    </b-row>
  </b-container>
</template>

<style></style>

<script>
import ModelCard from "@/components/ModelCard";

export default {
  name: "Nodes",
  data() {
    return {
      model_data: [],
      auth_token: "",
      error: false,
      cli_code: "git clone www.sybl.com/cli",
      cli_setup: "sybl-cli new",
    };
  },
  components: {
    ModelCard,
  },
  async mounted() {
    let user_id = $cookies.get("token");
    try {
      let data = await this.$http.get(
        `http://localhost:3001/api/clients/models`
      );
      this.model_data = data.data;
    } catch (err) {
      console.log(err);
    }
  },
  methods: {
    async copy(s) {
      await navigator.clipboard.writeText(s);
    },
  },
  computed: {
    validation() {
      return !this.error;
    },
  },
};
</script>
