<template>
  <b-container>
    <b-row>
      <b-col>
        <h1>Models</h1>
      </b-col>
    </b-row>

    <hr />
    <b-row class="justify-content-center mb-2">
      <b-col xs="12" lg="10">
        <h5>This is your private key please save this somewhere secure.</h5>
        <h5>
          This key will not be saved but you can request a new one anytime
          through the settings.
        </h5>
      </b-col>
    </b-row>
    <b-row class="justify-content-center mb-5">
      <b-col xs="12" lg="10">
        <b-card class="shadow">
          <b-row>
            <b-col md="10">
              <code class="rsaDisplay">{{ private_key }}</code>
            </b-col>
            <b-col style="text-align: right" md="2">
              <b-button variant="body">
                <b-icon-clipboard-plus
                  id="clipboard_btn"
                  @click="copy(private_key)"
                ></b-icon-clipboard-plus>
                <b-tooltip target="clipboard_btn" variant="primary" placement="right" triggers="click">Copied!</b-tooltip>
              </b-button>
            </b-col>
          </b-row>
        </b-card>
      </b-col>
    </b-row>
    <b-row class="justify-content-center">
      <b-col cols="1">
        <b-button variant="primary" @click="nodes"> Continue </b-button>
      </b-col>
    </b-row>
  </b-container>
</template>

<style scoped>
.rsaDisplay {
  user-select: none;
  white-space: pre-wrap;
}
</style>

<script>
export default {
  name: "KeyConfirmation",
  props: {
    private_key: String,
  },
  methods: {
    async copy(s) {
      await navigator.clipboard.writeText(s);
    },
    nodes() {
      this.$router.push({ name: "Models" });
    },
  },
};
</script>
