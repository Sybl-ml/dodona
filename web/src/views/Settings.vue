<template>
  <b-container fluid="md">
    <b-row>
      <b-col>
        <h1>Settings</h1>
        <hr>
        <h5>Change Avatar Icon:</h5>
        <avatar-upload @upload="onUpload"/>
        <b-button
          size="sm"
          variant="ready"
          type="submit"
          style="width:10rem"
          v-b-tooltip.hover
          @click="uploadAvatar"
          :disabled="!avatarSrc"
        >
          Update
        </b-button> 

        <br>
        <br>

        <h5>Regenerate Private Key:</h5>
        <b-button
          size="sm"
          variant="ready"
          type="submit"
          style="width:10rem"
          v-b-tooltip.hover
          @click="generatePrivateKey"
        >
          Generate
        </b-button> 
      </b-col>
    </b-row>
  </b-container>
</template>

<style>
</style>
<script>
import AvatarUpload from "@/components/AvatarUpload.vue";

export default {
  name: "Settings",
  data() {
    return {
      avatarSrc: "",
    }
  },
  components: {
    AvatarUpload,
  },
  methods: {
    onUpload(avatarSrc){
      this.avatarSrc = avatarSrc;
    },
    uploadAvatar(){
      if (this.avatarSrc) {
        this.$store.dispatch("postNewAvatar", this.avatarSrc.split(",")[1]);
      }
    },
	async generatePrivateKey() {
      let response = await this.$store.dispatch("generatePrivateKey");

      this.$router.push({
        name: "PrivateKey",
        params: { private_key: response.data.privKey },
      });
	},
  },
  computed: {
    user_data(){
      this.$store.dispatch("getUserData");
    }
  }
};
</script>
