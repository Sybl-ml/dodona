<template>
  <b-container fluid>
    <b-row class="justify-content-center text-center">
      <b-col lg="3" md="6" sm="8" xs="12">
        <br /><br />
        <icon-logo height="5em" width="5em" :show_text="false" />
        <h1>Sign In To Sybl</h1>
        <b-form class="mt-5 mb-3" @submit.prevent="onSubmit">
          <b-form-input
            class="mb-3"
            type="email"
            required
            placeholder="Enter Email"
            v-model="email"
          />

          <b-input-group class="mb-3">
            <b-form-input
              :type="passwordType"
              required
              placeholder="Password"
              v-model="password"
              pattern="^.{8,32}$"
              title="Password must contain at least eight characters"
            />
            <template #append>
              <b-input-group-text>
                <b-icon
                  style="cursor: pointer;"
                  :icon="passwordIcon"
                  @click="hidePassword = !hidePassword"
                />
              </b-input-group-text>
            </template>
          </b-input-group>

          <b-form-checkbox
            style="display:none;"
            value="me"
            class="mb-3"
            v-model="remember_password"
            >Remember Me</b-form-checkbox
          >
          <b-button variant="primary" type="submit" block>
            SIGN IN
            <b-spinner v-show="submitted" small></b-spinner>
          </b-button>
        </b-form>

        <a href="/forgot">Forgotten Password?</a>

        <br />
        <br />
      </b-col>
    </b-row>
    <b-row class="justify-content-center text-center">
      <b-alert v-model="failed" variant="danger" dismissible>
        Incorrect Username or Password
      </b-alert>
    </b-row>
  </b-container>
</template>

<script>
import IconLogo from "@/components/icons/IconLogo";
import axios from "axios";
export default {
  data() {
    return {
      email: "",
      password: "",
      remember_password: false,
      hidePassword: true,
      submitted: false,
      failed: false,
    };
  },
  components: {
    IconLogo,
  },
  computed: {
    passwordType() {
      return this.hidePassword ? "password" : "text";
    },
    passwordIcon() {
      return this.hidePassword ? "eye-fill" : "eye-slash-fill";
    },
  },
  methods: {
    sleep(ms) {
      return new Promise((resolve) => {
        setTimeout(resolve, ms);
      });
    },
    async onSubmit(e) {
      this.submitted = true;
      let response = await axios
        .post("http://localhost:3001/api/users/login", {
          email: this.email,
          password: this.password,
        })
        .catch((error) => {
          console.log(error.response.data.error);
        });

      if (response) {
        response = response.data;
        if (response.token === "null") {
          this.failed = true;
        } else {
          this.failed = false;
          $cookies.set("token", response.token, { path: "/", sameSite: true });
          this.$router.push("dashboard");
        }
      }
      await this.sleep(1000);
      this.failed = true;
      this.submitted = false;
    },
  },
};
</script>
