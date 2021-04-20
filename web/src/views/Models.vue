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
          variant="primary"
          style="border: none"
          onfocus="this.blur();"
          target="_blank" 
          href="https://www.notion.so/Become-a-Client-581ccdc92e2946159ad94e3dec44f72e"
        >
          <b-icon-plus-circle-fill></b-icon-plus-circle-fill>
          Add New Node
        </b-button>
      </b-col>
    </b-row>
    <!-- <b-row class="justify-content-center">
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
            <b-embed
              type="iframe"
              src="https://www.notion.so/Become-a-Client-3e39d2a2900a417eae259525216f8aec"
              ></b-embed>
          </b-card>
        </b-collapse>
      </b-col>
    </b-row> -->
  </b-container>
</template>

<style></style>

<script>
import ModelCard from "@/components/ModelCard";

export default {
  name: "Nodes",
  data() {
    return {
      auth_token: "",
      error: false,
      cli_code: "git clone www.sybl.com/cli",
      cli_setup: "sybl-cli new",
    };
  },
  components: {
    ModelCard,
  },
  async created() {
    this.$store.dispatch("getModels");
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
    model_data() {
      return this.$store.state.models;
    },
  },
};
</script>
